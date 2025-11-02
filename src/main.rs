use crate::database::{crypto::CryptoSuite, sqlite::SQLiteDB};
use actix_files::Files;
use actix_identity::IdentityMiddleware;
use actix_session::{SessionMiddleware, config::PersistentSession, storage::CookieSessionStore};
use actix_web::{App, HttpServer, middleware, web};
use anyhow::anyhow;
use clap::Parser;
use cookie::{Key, time::Duration};
use log::info;
use std::path::PathBuf;
use tokio::sync::RwLock;

mod database;
mod pages;
mod rest;

#[derive(clap::Parser, Clone, Debug)]
struct Cli {
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    fn startup_log(&self) {
        let mode = match self.command {
            Commands::Kiosk => "Demonstration",
            Commands::Run {
                password: _,
                salt: _,
                db_url: _,
            } => "Production",
        };
        let port = self.port;
        info!("Starting in {mode} mode on 'http://localhost:{port}'...");
    }
}

#[derive(clap::Subcommand, Clone, Debug)]
enum Commands {
    /// Run in demonstration mode (default mode for development)
    Kiosk,
    /// Run in production mode
    Run {
        /// File containing the password
        password: PathBuf,
        /// File containing the hash
        salt: PathBuf,
        /// Path to sqlite db
        #[arg(short, long, default_value_t = String::from("sqlite:.sqlite3"))]
        db_url: String,
    },
}

async fn run_user_facing_code(cli: Cli) -> anyhow::Result<()> {
    let (db, suite) = match cli.command {
        Commands::Kiosk => {
            let suite = CryptoSuite::new("demonstration_password", "demonstration_salt")
                .map_err(|e| anyhow!("Error: {e}"))?;
            (SQLiteDB::kiosk(&suite).await?, suite)
        }
        Commands::Run {
            password,
            salt,
            db_url,
        } => {
            let p = std::fs::read_to_string(password)?;
            let s = std::fs::read_to_string(salt)?;
            let suite = CryptoSuite::new(p.trim(), s.trim()).map_err(|e| anyhow!("Error: {e}"))?;
            let db = SQLiteDB::new(&db_url).await?;
            (db, suite)
        }
    };

    let wd = web::Data::new(RwLock::new(db));
    let pd = web::Data::new(suite);

    let secret_key = Key::generate();

    HttpServer::new(move || {
        App::new()
            .app_data(wd.clone())
            .app_data(pd.clone())
            .service(rest::create_services())
            .service(Files::new("/", "frontend/out").index_file("index.html"))
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_name("user_token".to_owned())
                    .cookie_secure(false)
                    .session_lifecycle(
                        PersistentSession::default().session_ttl(Duration::minutes(5)),
                    )
                    .build(),
            )
            .wrap(middleware::NormalizePath::trim())
            .wrap(middleware::Logger::default())
    })
    .bind(("0.0.0.0", cli.port))?
    .run()
    .await?;

    Ok(())
}

async fn run_backend_code(_cli: Cli) -> anyhow::Result<()> {
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder().parse_default_env().init();
    let cli = Cli::parse();
    cli.startup_log();
    let cli1 = cli.clone();
    let local = tokio::task::LocalSet::new();
    let ufc = local.run_until(async {
        let cli = cli1;
        tokio::task::spawn_local(run_user_facing_code(cli)).await
    });

    let backend = tokio::task::spawn(run_backend_code(cli));

    let handles = tokio::join!(ufc, backend);
    handles.0??;
    handles.1??;

    Ok(())
}
