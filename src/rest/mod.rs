use crate::database::{
    Database,
    crypto::{CryptData, CryptError, CryptoSuite},
    sqlite::*,
};
use actix_identity::Identity;
use actix_web::{
    HttpMessage, HttpRequest, HttpResponse, Responder, Result, get, post,
    web::{Data, Form, Json, Path, Query},
};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

pub fn create_services() -> actix_web::Scope {
    info!("Installing REST API services...");
    actix_web::web::scope("/api/chat")
        .service(login)
        .service(get_conversations)
        .service(get_peer)
        .service(get_user_profile)
        .service(get_message)
        .service(get_latest_message)
        .service(get_most_recent_messages)
        .service(start_conversation)
        .service(post_msg)
}

trait EasyLog<E> {
    fn log<F: FnOnce(&E)>(self, f: F) -> Self;
}

impl<T, E> EasyLog<E> for Result<T, E> {
    fn log<F: FnOnce(&E)>(self, f: F) -> Self {
        match &self {
            Ok(_) => {}
            Err(e) => f(e),
        }
        self
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Username {
    username: String,
}

#[get("/login")]
async fn login(
    data: Data<RwLock<SQLiteDB>>,
    name: Query<Username>,
    req: HttpRequest,
) -> Result<impl Responder> {
    // TODO,FIXME: Add auth (dies inside)

    let username = name.username.clone();

    // FIXME: This forbids new users from login, basically.
    data.read()
        .await
        .get_user_id_from_username(&username)
        .await
        .log(|e| warn!("Error: '{e}'; Possibly this user is unregistered?"))?;

    // Attach identity
    Identity::login(&req.extensions(), username)?;

    Ok(HttpResponse::Ok())
}

// FIXME: usr_id needs be usr_token
#[get("/conversation")]
async fn get_conversations(user: Identity, data: Data<RwLock<SQLiteDB>>) -> Result<impl Responder> {
    let username = user.id()?;
    let usr_id = data
        .read()
        .await
        .get_user_id_from_username(&username)
        .await
        .log(|e| warn!("{e}"))?;
    let res = data
        .read()
        .await
        .get_conversations(&usr_id)
        .await
        .map(Json)
        .log(|e| warn!("{e}"))?;
    Ok(res)
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
        .await
        .log(|e| warn!("{e}"))?;
    let convo_id = ConversationId(convo_id.clone());
    data.read()
        .await
        .belongs_to_conversation(&usr_id, &convo_id)
        .await
        .log(|e| warn!("{e}"))?;
    let peer_id = data.read().await.get_peer(&usr_id, &convo_id).await?;
    let username = data.read().await.get_user_profile(&peer_id).await?;
    Ok(Json(username.username()))
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
        .map_err(DbError::from)
        .log(|e| warn!("{e}"))?;
    let res = data
        .read()
        .await
        .get_user_profile(&usr_id)
        .await
        .map(Json)
        .log(|e| warn!("{e}"))?;
    Ok(res)
}

#[derive(Debug, Serialize, Deserialize)]
struct MessageContent {
    sender_username: String,
    msg: Message,
}

#[derive(Debug, Serialize, Deserialize)]
enum RequestContents {
    #[serde(untagged)]
    One(MessageContent),
    #[serde(untagged)]
    Many(Vec<MessageContent>),
}

#[derive(Debug, Serialize, Deserialize)]
struct MessageFormat {
    content: RequestContents,
    previous_msg: Option<MessageId>,
}

impl MessageContent {
    fn new(sender_username: String, msg: Message) -> Self {
        Self { sender_username, msg }
    }
}

impl MessageFormat {
    fn one(msg: MessageContent, prev_id: Option<MessageId>) -> Self {
        Self {
            content: RequestContents::One(msg),
            previous_msg: prev_id,
        }
    }

    fn many(msgs: Vec<MessageContent>, prev_id: Option<MessageId>) -> Self {
        Self {
            content: RequestContents::Many(msgs),
            previous_msg: prev_id,
        }
    }
}

#[get("/message/{msg_id}")]
async fn get_message(
    data: Data<RwLock<SQLiteDB>>,
    suite: Data<CryptoSuite>,
    user: Identity,
    msg_id: Path<i64>,
) -> Result<impl Responder> {
    let username = user.id()?;
    let usr_id = data
        .read()
        .await
        .get_user_id_from_username(&username)
        .await
        .log(|e| warn!("{e}"))?;
    let msg_id = MessageId(msg_id.clone());
    let convo_id = data
        .read()
        .await
        .get_conversation_from_message(&msg_id)
        .await
        .log(|e| warn!("{e}"))?;
    data.read()
        .await
        .belongs_to_conversation(&usr_id, &convo_id)
        .await
        .log(|e| warn!("{e}"))?;
    let (sender_id, encrypted_msg, prev_id) = data
        .read()
        .await
        .get_message(&msg_id)
        .await
        .log(|e| warn!("{e}"))?;
    let msg = encrypted_msg.decrypt(&suite).log(|e| warn!("{e}"))?;
    let sender_username = data.read().await.get_user_profile(&sender_id).await.log(|e| warn!("{e}"))?.username();
    let msg = MessageContent::new(sender_username, msg);
    Ok(Json(MessageFormat::one(msg, prev_id)))
}

// #[post("/user")]
// async fn add_user(
//     data: Data<RwLock<SQLiteDB>>,
//     user_profile: Form<UserProfile>,
// ) -> Result<impl Responder> {
//     let user_profile = user_profile.0;
//     Ok(data.write().await.add_user(&user_profile).await.map(Json)?)
// }

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
struct ConversationForm {
    their_username: String,
}

#[post("/conversation")]
async fn start_conversation(
    data: Data<RwLock<SQLiteDB>>,
    user: Identity,
    form: Form<ConversationForm>,
) -> Result<impl Responder> {
    let username = user.id()?;
    let usr_id = data
        .read()
        .await
        .get_user_id_from_username(&username)
        .await
        .log(|e| warn!("{e}"))?;
    let their_id = data
        .read()
        .await
        .get_user_id_from_username(&form.their_username)
        .await
        .log(|e| warn!("{e}"))?;
    let res = data
        .write()
        .await
        .start_conversation(&usr_id, &their_id)
        .await
        .map(Json)
        .log(|e| warn!("{e}"))?;
    Ok(res)
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
struct MessageForm {
    message: String,
}

#[post("/conversation/{convo_id}/message")]
async fn post_msg(
    data: Data<RwLock<SQLiteDB>>,
    suite: Data<CryptoSuite>,
    user: Identity,
    conversation: Path<i64>,
    form: Form<MessageForm>,
) -> Result<impl Responder> {
    let username = user.id()?;
    let usr_id = data
        .read()
        .await
        .get_user_id_from_username(&username)
        .await
        .log(|e| warn!("{e}"))?;
    let conversation = ConversationId(conversation.into_inner());
    let msg = CryptData::encrypt(Message(form.message.clone()), &suite).log(|e| warn!("{e}"))?;
    data.read()
        .await
        .belongs_to_conversation(&usr_id, &conversation)
        .await
        .log(|e| warn!("{e}"))?;
    let res = data
        .write()
        .await
        .post_msg(msg, &usr_id, &conversation)
        .await
        .map(Json)
        .log(|e| warn!("{e}"))?;
    Ok(res)
}

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
        .await
        .log(|e| warn!("{e}"))?;
    let convo_id = ConversationId(convo_id.clone());
    data.read()
        .await
        .belongs_to_conversation(&usr_id, &convo_id)
        .await
        .log(|e| warn!("{e}"))?;
    let db_handle = data.read().await;
    let res = db_handle
        .get_latest_message(&convo_id)
        .await
        .map(Json)
        .log(|e| warn!("{e}"))?;
    Ok(res)
}

#[get("/conversation/{convo_id}/recent")]
async fn get_most_recent_messages(
    data: Data<RwLock<SQLiteDB>>,
    suite: Data<CryptoSuite>,
    user: Identity,
    convo_id: Path<i64>,
) -> Result<impl Responder> {
    let username = user.id()?;
    let usr_id = data
        .read()
        .await
        .get_user_id_from_username(&username)
        .await
        .log(|e| warn!("{e}"))?;
    let convo_id = ConversationId(convo_id.clone());
    data.read()
        .await
        .belongs_to_conversation(&usr_id, &convo_id)
        .await
        .log(|e| warn!("{e}"))?;
    let db_handle = data.read().await;
    let (messages, prev_id) = db_handle
        .get_most_recent_messages(&convo_id)
        .await
        .log(|e| warn!("{e}"))?;
    let mut msgs = Vec::new();
    for (sender_id, encrypted) in messages {
        let msg = encrypted.decrypt(&suite).log(|e| warn!("{e}"))?;
        let sender_username = data.read().await.get_user_profile(&sender_id).await.log(|e| warn!("{e}"))?.username();
        msgs.push(MessageContent { sender_username, msg })
    }
    Ok(Json(MessageFormat::many(msgs, prev_id)))
}
