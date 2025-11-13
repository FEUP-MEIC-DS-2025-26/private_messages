use crate::database::{Database, sqlite::*};
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
        // DONE: Doc'ed
        .service(login)
        // DONE: Doc'ed
        .service(get_conversations)
        // DONE: Doc'ed
        .service(get_peer)
        // DONE: Doc'ed
        .service(get_user_profile)
        // DONE: Doc'ed
        .service(get_message)
        // DONE: Doc'ed
        .service(get_latest_message)
        // DONE: Doc'ed
        .service(get_most_recent_messages)
        // DONE: Doc'ed
        .service(start_conversation)
        // DONE: Doc'ed
        .service(post_msg)
        // DONE: Doc'ed
        .service(add_product)
        // DONE: Doc'ed
        .service(get_product)
        // DONE: Doc'ed
        .service(get_product_in_conversation)
        .default_service(actix_web::web::to(default_service))
}

async fn default_service() -> impl Responder {
    HttpResponse::NotFound().content_type("text/html").body(r#"
        <html>
            <style>
                h1 {
                    text-align: center; /* Center the text */
                    font-size: 3em; /* Make the text larger */
                    margin: 0 auto; /* Center the block */
                    width: 80%; /* Optional: set a width */
                }
            </style>
            <h1>404: Not Found</h1>
            Try:
                 /api/chat
                          ⊢ /login                             ---> Enables internal cookie.
                          ⊢ /conversation                      ---> (GET) Lists conversations a user is in. (POST) Starts a conversation.
                                         ⊢ /{convo_id}/peer    ---> Gets the username of the peer.
                                         ⊢ /{convo_id}/latest  ---> Gets the latest message.
                                         ⊢ /{convo_id}/recent  ---> Gets the 32 most recent messages.
                                         ⊢ /{convo_id}/product ---> Gets the product associated with the conversation.
                                         ⊢ /{convo_id}/message ---> Posts a new message into the chat.
                          ⊢ /message/{msg_id}                  ---> Gets the message with ID 'msg_id'.
                          ⊢ /user/{username}                   ---> Gets the profile of user with username 'username'.
                          ⊢ /product                           ---> Posts a new product into the database.
                          ⊢ /product/{prod_id}                 ---> Gets the product with id 'prod_id'.
        </html>
         "#)
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
    let convo_id = ConversationId(*convo_id);
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
        Self {
            sender_username,
            msg,
        }
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
    let msg_id = MessageId(*msg_id);
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
    let (sender_id, msg, prev_id) = data
        .read()
        .await
        .get_message(&msg_id)
        .await
        .log(|e| warn!("{e}"))?;
    let sender_username = data
        .read()
        .await
        .get_user_profile(&sender_id)
        .await
        .log(|e| warn!("{e}"))?
        .username();
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
    product_id: i64,
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
    data.read()
        .await
        .belongs_to_seller(&their_id, &form.product_id.into())
        .await
        .log(|e| warn!("{e}"))?;
    let res = data
        .write()
        .await
        .start_conversation(&usr_id, &their_id, &form.product_id.into())
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
    let msg = Message(form.into_inner().message);
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
    let convo_id = ConversationId(*convo_id);
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
    let convo_id = ConversationId(*convo_id);
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
    for (sender_id, msg) in messages {
        let sender_username = data
            .read()
            .await
            .get_user_profile(&sender_id)
            .await
            .log(|e| warn!("{e}"))?
            .username();
        msgs.push(MessageContent {
            sender_username,
            msg,
        })
    }
    Ok(Json(MessageFormat::many(msgs, prev_id)))
}

#[get("/product/{prod_id}")]
async fn get_product(data: Data<RwLock<SQLiteDB>>, prod_id: Path<i64>) -> Result<impl Responder> {
    let prod = data.read().await.get_product(&ProductId(*prod_id)).await?;
    Ok(Json(prod))
}

#[get("/conversation/{convo_id}/product")]
async fn get_product_in_conversation(
    data: Data<RwLock<SQLiteDB>>,
    convo_id: Path<i64>,
    user: Identity,
) -> Result<impl Responder> {
    let username = user.id()?;
    let usr_id = data
        .read()
        .await
        .get_user_id_from_username(&username)
        .await
        .log(|e| warn!("{e}"))?;
    data.read()
        .await
        .belongs_to_conversation(&usr_id, &ConversationId(*convo_id))
        .await?;
    let prod = data
        .read()
        .await
        .get_product_id_from_conversation_id(&ConversationId(*convo_id))
        .await?;
    Ok(Json(prod))
}

#[derive(Debug, Serialize, Deserialize)]
struct ImportProductForm {
    jumpseller_id: i64,
    seller_id: i64,
    name: String,
}

impl From<ImportProductForm> for Product {
    fn from(value: ImportProductForm) -> Self {
        Self::new(value.name, UserId(value.seller_id), value.jumpseller_id)
    }
}

#[post("/product")]
async fn add_product(
    data: Data<RwLock<SQLiteDB>>,
    form: Form<ImportProductForm>,
) -> Result<impl Responder> {
    let product = form.0.into();
    Ok(data.write().await.add_product(&product).await.map(Json)?)
}
