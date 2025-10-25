use actix_files::Files;
use actix_web::{App, HttpServer};

mod grpc;
mod pages;
mod rest;

async fn run_user_facing_code() -> anyhow::Result<()> {
    HttpServer::new(|| {
        App::new().service(Files::new("/", "frontend/out").index_file("index.html"))
    })
    .bind(("0.0.0.0", 8080))?
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
