use crate::{database::sqlite::SQLiteDB, rest::*};
use actix_files::Files;
use actix_web::{App, HttpServer, web};
use clap::Parser;
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

    HttpServer::new(move || {
        App::new()
            .app_data(wd.clone())
            .service(get_conversations)
            .service(get_peer)
            .service(get_user_profile)
            .service(get_message)
            .service(add_user)
            // .service(start_conversation)
            // .service(post_msg)
            .service(get_latest_message)
            .service(Files::new("/", "frontend/out").index_file("index.html"))
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
