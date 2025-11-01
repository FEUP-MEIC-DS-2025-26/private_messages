use std::path::PathBuf;

use actix_web::{get, HttpResponse, Result};
use actix_files::NamedFile;
use std::fs;

#[get("/")]
pub async fn index() -> Result<NamedFile> {
    let path: PathBuf = "frontend/out/index.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

#[get("/cat")]
pub async fn cat() -> HttpResponse {
    let messages = vec![
        (0, "Hello There"),
        (1, "General Kenobi"),
        (0, "I'm glad"),
        (0, "I like cake"),
        (1, "Me too")
    ];

    let mut html_messages = String::with_capacity(messages.len() * 32);
    let chat_html = fs::read_to_string("templates/chat.html").expect("Oh no!");

    for (id, text) in messages {
        let align = if id == 0 { "left" } else { "right" };
        html_messages.push_str(&format!("<li style=\"width: 50%; text-align: {align}\">{text}</li>").to_string());
    }

    let chat_html = chat_html.replace("$chat", &html_messages);
    HttpResponse::Ok().body(chat_html)
}

// This means that we should have a database in our prototype???
// ohs
