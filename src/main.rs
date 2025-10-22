use actix_files::Files;
use actix_web::{App, HttpServer};

mod grpc;
mod pages;
mod rest;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(Files::new("/", "frontend/out").index_file("index.html")))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
