use crate::database::{
    Database,
    crypto::CryptoKey,
    sqlite::{ConversationId, DbError, MessageId, SQLiteDB},
};
use actix_files::Files;
use actix_identity::IdentityMiddleware;
use actix_session::{SessionMiddleware, config::PersistentSession, storage::CookieSessionStore};
use actix_web::{App, HttpServer, middleware, web};
use anyhow::anyhow;
use clap::Parser;
use cookie::{Key, time::Duration};
use gcloud_pubsub::client::{Client, ClientConfig};
use log::info;
use serde::{Deserialize, Serialize};
use std::{ffi::OsString, fmt::Debug, path::PathBuf};
use tokio::sync::RwLock;

use tokio::time::Duration as TDuration;

mod database;
mod rest;
mod jumpseller;

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
                jumpseller_cred_file:_,
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
        /// File containing json for the JumpSeller credentials.
        #[arg(default_value = OsString::from("local/jumpseller_cred.json"))]
        jumpseller_cred_file: PathBuf,
        /// Path to sqlite db
        #[arg(short, long, default_value_t = String::from("sqlite:.sqlite3"))]
        db_url: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JumpSellerCredentials {
    pub(crate) login : String,
    pub(crate) token: String,
}

async fn run_user_facing_code(cli: Cli, utils: BackendInfoUpdater) -> anyhow::Result<()> {
    let (db, js_cred) = match cli.command {
        Commands::Kiosk => {
            let suite = CryptoKey::new("demonstration_password", "demonstration_salt")
                .map_err(|e| anyhow!("Error: {e}"))?;
            (SQLiteDB::kiosk(suite).await?, None)
        }
        Commands::Run {
            password,
            salt,
            db_url,
            jumpseller_cred_file
        } => {
            let p = std::fs::read_to_string(password)?;
            let s = std::fs::read_to_string(salt)?;
            let suite = CryptoKey::new(p.trim(), s.trim()).map_err(|e| anyhow!("Error: {e}"))?;
            let js_f: Option<JumpSellerCredentials> = std::fs::read_to_string(jumpseller_cred_file).ok().and_then(|x| serde_json::from_str(&x).ok());

            (SQLiteDB::new(&db_url, suite).await?, js_f)
        }
    };

    let js_client = match js_cred {
        Some(s) => jumpseller::Client::from(s),
        None => panic!("[FAIL] Jumpseller credential file not found or invalid."),
    };

    let jsc = web::Data::new(js_client);

    let wd = web::Data::new(RwLock::new(db));

    let utils = web::Data::new(utils);

    let secret_key = Key::generate();

    HttpServer::new(move || {
        App::new()
            .app_data(utils.clone())
            .app_data(wd.clone())
            .app_data(jsc.clone())
            .service(rest::create_services())
            // .service(Files::new("/", "frontend/dist").index_file("index.html"))
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

async fn run_backend_code(
    _cli: Cli,
    mut receiver: tokio::sync::mpsc::Receiver<F2BRequest>,
) -> anyhow::Result<()> {
    // let gcloud_ep = Client::new(ClientConfig::default()).await?;
    while let Some(F2BRequest { msg, callback }) = receiver.recv().await {
        match msg {
            F2BRequestType::NewMessage {
                sender_name,
                receiver_name,
                product_info,
                contents,
            } => {
                log::error!("BACKEND IS UNIMPLEMENTED.");
                _ = callback.send(F2BResponse::Ok);
            }
        }
    }
    Ok(())
}

pub enum F2BResponse {
    Ok,
    GoogleCloud(gcloud_pubsub::client::Error),
    Unrecoverable(anyhow::Error),
}

enum F2BRequestType {
    NewMessage {
        sender_name: String,
        receiver_name: String,
        /// jumpseller id
        product_info: i64,
        contents: [char; 32],
    },
}

struct F2BRequest {
    msg: F2BRequestType,
    callback: tokio::sync::oneshot::Sender<F2BResponse>,
}

pub struct BackendInfoUpdater(tokio::sync::mpsc::Sender<F2BRequest>);
type CallBack = tokio::sync::oneshot::Receiver<F2BResponse>;

impl BackendInfoUpdater {
    pub async fn new_message(
        &self,
        database: &SQLiteDB,
        message_id: &MessageId,
        convo_id: &ConversationId,
    ) -> Result<CallBack, DbError> {
        let (s, r) = tokio::sync::oneshot::channel();
        let (sender, message, _) = database.get_message(message_id).await?;
        let receiver = database.get_peer(&sender, convo_id).await?;

        let sender_name = database.get_user_profile(&sender).await?.username();
        let receiver_name = database.get_user_profile(&receiver).await?.username();
        let product_id = database
            .get_product_id_from_conversation_id(convo_id)
            .await?;
        let product = database.get_product(&product_id).await?;
        let product_info = product.product_info();
        let mut message_sum = [char::default(); 32];
        let fst_32 = message.0.chars().take(32).collect::<Vec<_>>();
        message_sum[..fst_32.len()].copy_from_slice(&fst_32);

        let msg_type = F2BRequestType::NewMessage {
            sender_name,
            receiver_name,
            product_info,
            contents: message_sum,
        };

        let msg = F2BRequest {
            msg: msg_type,
            callback: s,
        };

        _ = self.0.send(msg).await;

        Ok(r)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder().parse_default_env().init();
    let cli = Cli::parse();
    cli.startup_log();
    let cli1 = cli.clone();

    let (tcv, rcv) = tokio::sync::mpsc::channel::<F2BRequest>(10);
    let frontend_util = BackendInfoUpdater(tcv);

    let local = tokio::task::LocalSet::new();
    let ufc = local.run_until(async {
        let cli = cli1;
        tokio::task::spawn_local(run_user_facing_code(cli, frontend_util)).await
    });

    let backend = tokio::task::spawn(run_backend_code(cli, rcv));

    let handles = tokio::join!(ufc, backend);
    handles.0??; // no way bro!
    handles.1??; // do you even hear yourself bro?

    Ok(())
}
