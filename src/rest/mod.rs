use crate::database::{
    Database,
    crypto::{CryptData, CryptoSuite},
    sqlite::*,
};
use actix_identity::Identity;
use actix_web::{
    HttpMessage, HttpRequest, HttpResponse, Responder, Result, get, post,
    web::{Data, Form, Json, Path, Query},
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

pub fn create_services() -> actix_web::Scope {
    actix_web::web::scope("/api/chat")
        .service(login)
        .service(get_conversations)
        .service(get_peer)
        .service(get_user_profile)
        .service(get_message)
        .service(get_latest_message)
}

#[derive(Debug, Serialize, Deserialize)]
struct Username {
    username: String,
}

#[get("/login")]
async fn login(name: Query<Username>, req: HttpRequest) -> Result<impl Responder> {
    // TODO,FIXME: Add auth (dies inside)

    let username = name.username.clone();
    // Attach identity
    Identity::login(&req.extensions(), username)?;

    Ok(HttpResponse::Found())
}

// FIXME: usr_id needs be usr_token
#[get("/conversation")]
async fn get_conversations(user: Identity, data: Data<RwLock<SQLiteDB>>) -> Result<impl Responder> {
    let username = user.id()?;
    let usr_id = data
        .read()
        .await
        .get_user_id_from_username(&username)
        .await?;
    Ok(data
        .read()
        .await
        .get_conversations(&usr_id)
        .await
        .map(Json)?)
}

// FIXME: usr_id needs be usr_token
#[get("/conversation/{convo_id}/peer")]
async fn get_peer(
    data: Data<RwLock<SQLiteDB>>,
    user: Identity,
    convo_id: Path<i64>,
) -> Result<impl Responder> {
    let username = user.id()?;
    let usr_id = data
        .read()
        .await
        .get_user_id_from_username(&username)
        .await?;
    let convo_id = ConversationId(convo_id.clone());
    data.read()
        .await
        .belongs_to_conversation(&usr_id, &convo_id)
        .await?;
    Ok(data
        .read()
        .await
        .get_peer(&usr_id, &convo_id)
        .await
        .map(Json)?)
}

#[get("/user/{username}")]
async fn get_user_profile(
    data: Data<RwLock<SQLiteDB>>,
    username: Path<String>,
) -> Result<impl Responder> {
    let usr_id = data
        .read()
        .await
        .get_user_id_from_username(&username)
        .await
        .map_err(DbError::from)?;
    Ok(data
        .read()
        .await
        .get_user_profile(&usr_id)
        .await
        .map(Json)?)
}

// FIXME: usr_id needs be usr_token
#[get("/message/{msg_id}")]
async fn get_message(
    data: Data<RwLock<SQLiteDB>>,
    user: Identity,
    msg_id: Path<i64>,
) -> Result<impl Responder> {
    let username = user.id()?;
    let usr_id = data
        .read()
        .await
        .get_user_id_from_username(&username)
        .await?;
    let msg_id = MessageId(msg_id.clone());
    let msg = data.read().await.get_message(&msg_id).await?;
    let convo_id = data
        .read()
        .await
        .get_conversation_from_message(&msg_id)
        .await?;
    data.read()
        .await
        .belongs_to_conversation(&usr_id, &convo_id)
        .await?;
    Ok(Json(msg))
}

// #[post("/user")]
// async fn add_user(
//     data: Data<RwLock<SQLiteDB>>,
//     user_profile: Form<UserProfile>,
// ) -> Result<impl Responder> {
//     let user_profile = user_profile.0;
//     Ok(data.write().await.add_user(&user_profile).await.map(Json)?)
// }

// FIXME: usr_id needs be usr_token
#[post("/conversation")]
async fn start_conversation(
    data: Data<RwLock<SQLiteDB>>,
    user: Identity,
    their_id: Form<UserId>,
) -> Result<impl Responder> {
    let username = user.id()?;
    let usr_id = data
        .read()
        .await
        .get_user_id_from_username(&username)
        .await?;
    let their_id = their_id.0;
    Ok(data
        .write()
        .await
        .start_conversation(&usr_id, &their_id)
        .await
        .map(Json)?)
}

// FIXME: usr_id needs be usr_token
#[post("/conversation/{convo_id}/message")]
async fn post_msg(
    data: Data<RwLock<SQLiteDB>>,
    suite: Data<CryptoSuite>,
    user: Identity,
    conversation: Path<i64>,
    msg: Form<Message>,
) -> Result<impl Responder> {
    let username = user.id()?;
    let usr_id = data
        .read()
        .await
        .get_user_id_from_username(&username)
        .await?;
    let conversation = ConversationId(conversation.into_inner());
    let msg = msg.0;
    let msg = CryptData::encrypt(msg, &suite)?;
    data.read()
        .await
        .belongs_to_conversation(&usr_id, &conversation)
        .await?;
    Ok(data
        .write()
        .await
        .post_msg(msg, &usr_id, &conversation)
        .await
        .map(Json))
}

// FIXME: usr_id needs be usr_token
#[get("/conversation/{convo_id}/latest")]
async fn get_latest_message(
    data: Data<RwLock<SQLiteDB>>,
    user: Identity,
    convo_id: Path<i64>,
) -> Result<impl Responder> {
    let username = user.id()?;
    let usr_id = data
        .read()
        .await
        .get_user_id_from_username(&username)
        .await?;
    let convo_id = ConversationId(convo_id.clone());
    data.read()
        .await
        .belongs_to_conversation(&usr_id, &convo_id)
        .await?;
    let db_handle = data.read().await;
    Ok(db_handle.get_latest_message(&convo_id).await.map(Json)?)
}
