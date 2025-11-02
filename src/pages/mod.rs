use std::path::PathBuf;

use actix_web::{get, Result};
use actix_files::NamedFile;

#[get("/")]
pub async fn index() -> Result<NamedFile> {
    let path: PathBuf = "frontend/out/index.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}
