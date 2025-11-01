use crate::database::sqlite::SQLiteDB;
use actix_files::Files;
use actix_identity::IdentityMiddleware;
use actix_session::{SessionMiddleware, config::PersistentSession, storage::CookieSessionStore};
use actix_web::{App, HttpServer, middleware, web};
use clap::Parser;
use cookie::{Key, time::Duration};
use tokio::sync::RwLock;

mod database;
mod pages;
mod rest;

#[derive(clap::Parser)]
struct Cli {
    #[arg(short, long, default_value_t = String::from("sqlite:.sqlite3"))]
    db_url: String,
    #[arg(short, long, default_value_t = 8080)]
    port: u16,
}

async fn run_user_facing_code() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let db = SQLiteDB::new(&cli.db_url).await?;
    let wd = web::Data::new(RwLock::new(db));
    let secret_key = Key::generate();

    HttpServer::new(move || {
        App::new()
            .app_data(wd.clone())
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

async fn run_backend_code() -> anyhow::Result<()> {
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let local = tokio::task::LocalSet::new();
    let ufc = local.run_until(async { tokio::task::spawn_local(run_user_facing_code()).await });

    let backend = tokio::task::spawn(run_backend_code());

    let handles = tokio::join!(ufc, backend);
    handles.0??;
    handles.1??;

    Ok(())
}
