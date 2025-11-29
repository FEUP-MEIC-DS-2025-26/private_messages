#![warn(clippy::pedantic)]
#![warn(clippy::perf)]
#![deny(clippy::correctness)]
#![deny(clippy::panicking_unwrap)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use crate::database::{
    Database,
    crypto::CryptoKey,
    sqlite::{ConversationId, DbError, MessageId, SQLiteDB, UserId},
};
use actix_identity::IdentityMiddleware;
use actix_session::{SessionMiddleware, config::PersistentSession, storage::CookieSessionStore};
use actix_web::{App, HttpServer, middleware, web};
use anyhow::anyhow;
use clap::Parser;
use cookie::{Key, time::Duration};
use gcloud_googleapis::pubsub::v1::PubsubMessage;
use gcloud_pubsub::client::{Client, ClientConfig};
use log::info;
use serde::{Deserialize, Serialize};
use std::{ffi::OsString, fmt::Debug, path::PathBuf};
use prost::Message;
use tokio::sync::RwLock;

mod database;
mod pubsub;
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

async fn handle_pubsub_failure_state(mut receiver: tokio::sync::mpsc::Receiver<F2BRequest>) -> ! {
    loop {
        let lost_req = receiver.recv().await.map(|x| {
            _ = x.callback.send(F2BResponse::Ok);
            x.msg
        });
        log::warn!("Lost request {lost_req:?} because GCloud feature is disabled.");
        if lost_req.is_none() {
            std::process::exit(0);
        }
    }
}

async fn run_backend_code(
    _cli: Cli,
    mut receiver: tokio::sync::mpsc::Receiver<F2BRequest>,
) -> anyhow::Result<()> {
    let debug_env = std::env::var("PUBSUB_EMULATOR_HOST");

    let config = if debug_env.is_ok() {
        log::info!("Starting with Emulator...");
        ClientConfig::default()
    } else {
        log::info!("Starting with GCloud...");
        match ClientConfig::default().with_auth().await {
            Ok(c) => c,
            Err(e) => {
                log::error!(
                    "Failed to initialize gcloud config, disabling feature... (Reason: {e})"
                );
                handle_pubsub_failure_state(receiver).await;
            }
        }
    };

    let gcloud_ep = match Client::new(config).await {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to initialize gcloud endpoint, disabling feature... (Reason: {e})");
            handle_pubsub_failure_state(receiver).await
        }
    };
    let private_messages_topic = gcloud_ep.topic("projects/ds-2526-mips/topics/private_messages");
    if !private_messages_topic.exists(None).await.unwrap_or(false) {
        match private_messages_topic.create(None, None).await {
            Ok(()) => {}
            Err(e) => {
                log::error!("Failed to create topic with {e}. Disabling pubsub...");
                handle_pubsub_failure_state(receiver).await;
            }
        }
    }

    let pm_publisher = private_messages_topic.new_publisher(None);

    while let Some(F2BRequest { msg, callback }) = receiver.recv().await {
        use pubsub::priv_msgs_v1::{PrivateMessageSchema, private_message_schema};
        let pubsub_msg = match msg {
            F2BRequestType::NewMessage {
                sender_name,
                receiver_name,
                product_info,
                uid,
                timestamp,
                preview,
            } => {
                let pubsub_msg = private_message_schema::NewMessage {
                    uid,
                    sender_name,
                    receiver_name,
                    product_info,
                    timestamp,
                    preview,
                };

                let pubsub_msg = PrivateMessageSchema {
                    contents: Some(private_message_schema::Contents::NewMessage(pubsub_msg)),
                };

                PubsubMessage {
                    data: pubsub_msg.encode_to_vec(),
                    ..Default::default()
                }
            }
            F2BRequestType::NewConvo {
                uid,
                seller,
                buyer,
                product_info,
            } => {
                let pubsub_msg = private_message_schema::NewConversation {
                    uid,
                    seller_name: seller,
                    buyer_name: buyer,
                    product_info,
                };

                let pubsub_msg = PrivateMessageSchema {
                    contents: Some(private_message_schema::Contents::NewConversation(
                        pubsub_msg,
                    )),
                };

                PubsubMessage {
                    data: pubsub_msg.encode_to_vec(),
                    ..Default::default()
                }
            }
        };
        let waiter = pm_publisher.publish(pubsub_msg).await.get().await;

        match waiter {
            Ok(s) => {
                log::info!("Success: '{s}'.");
                _ = callback.send(F2BResponse::Ok);
            }
            // TODO: Handle this error
            Err(e) => {
                log::error!("Failure: '{e}'.");
                _ = callback.send(F2BResponse::Unrecoverable(e.into()));
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

#[derive(Debug)]
enum F2BRequestType {
    #[allow(dead_code)]
    NewMessage {
        uid: i64,
        sender_name: String,
        receiver_name: String,
        /// jumpseller id
        product_info: i64,
        timestamp: String,
        preview: Option<String>,
    },
    NewConvo {
        uid: i64,
        seller: String,
        buyer: String,
        /// jumpseller id
        product_info: i64,
    },
}

struct F2BRequest {
    msg: F2BRequestType,
    callback: tokio::sync::oneshot::Sender<F2BResponse>,
}

pub struct BackendInfoUpdater(tokio::sync::mpsc::Sender<F2BRequest>);
type CallBack = tokio::sync::oneshot::Receiver<F2BResponse>;

impl BackendInfoUpdater {
    /// # Errors
    /// This function may fail if the Database state is buggy or when the database has a bug
    pub async fn new_message(
        &self,
        database: &SQLiteDB,
        message_id: &MessageId,
        convo_id: &ConversationId,
        divulge: bool,
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
        let fst_32 = message.contents().chars().take(32).collect::<String>();

        let message_sum = if divulge { Some(fst_32) } else { None };

        let msg_type = F2BRequestType::NewMessage {
            sender_name,
            receiver_name,
            product_info,
            preview: message_sum,
            uid: message_id.0,
            timestamp: message.timestamp().to_string(),
        };

        let msg = F2BRequest {
            msg: msg_type,
            callback: s,
        };

        _ = self.0.send(msg).await;

        Ok(r)
    }
    /// # Errors
    /// This function may fail if the Database state is buggy or when the database has a bug
    pub async fn new_convo(
        &self,
        database: &SQLiteDB,
        convo_id: &ConversationId,
        buyer: &UserId,
    ) -> Result<CallBack, DbError> {
        let (s, r) = tokio::sync::oneshot::channel();
        let prod_id = database
            .get_product_id_from_conversation_id(convo_id)
            .await?;
        let prod = database.get_product(&prod_id).await?;
        let product_info = prod.product_info();
        let seller_id = database.get_peer(buyer, convo_id).await?;
        let buyer = database.get_user_profile(buyer).await?.username();
        let seller = database.get_user_profile(&seller_id).await?.username();

        let msg_type = F2BRequestType::NewConvo {
            uid: convo_id.0,
            seller,
            buyer,
            product_info,
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
