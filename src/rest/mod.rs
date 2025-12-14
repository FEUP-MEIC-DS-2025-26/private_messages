use std::num::ParseIntError;

use crate::{
    BackendInfoUpdater, IsProd,
    database::{
        Database,
        sqlite::{
            ConversationId, DbError, Message, MessageId, Product, ProductId, SQLiteDB, UserId,
        },
    },
    jumpseller::{self, JumpSellerErr},
};
use actix_identity::Identity;
use actix_web::{
    HttpMessage, HttpRequest, HttpResponse, Responder, ResponseError, Result,
    error::ErrorInternalServerError,
    get, post,
    web::{Data, Form, Json, Path, Query},
};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

async fn jumpseller_update_product(
    db: &RwLock<SQLiteDB>,
    js: &jumpseller::Client,
    seller_id: &UserId,
    product_id: i64,
) -> Result<(), DbError> {
    let p = js.get_product(product_id).await.w();
    match p {
        Ok(ref prod) => {
            db.write()
                .await
                .add_product(&Product::new(prod.name.clone(), *seller_id, prod.id))
                .await
                .w()?;
        }
        Err(JumpSellerErr::ResponseErr(_, Some(reqwest::StatusCode::NOT_FOUND))) => {
            log::warn!("Jumpseller has no product with ID {product_id}.");
            // TODO: delete products that don't exist anymore, if no more conversations mentioning it exist.
        }
        Err(ref e) => {
            log::error!("Failed to get product from Jumpseller: {e}");
        }
    }
    Ok(())
}

async fn jumpseller_update_user(
    db: &RwLock<SQLiteDB>,
    js: &jumpseller::Client,
    user_id: i64,
) -> Result<(), DbError> {
    if let Ok(profile) = js.get_user(user_id).await.w() {
        db.write().await.add_user(&profile).await.w()?;
    }
    Ok(())
}

pub fn create_services() -> actix_web::Scope {
    info!("Installing REST API services...");
    actix_web::web::scope("/api/chat")
        // DONE: Doc'ed
        .service(login)
        // DONE: Doc'ed
        .service(me)
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
                
                body {
                    display: flex;
                    margin: 0;
                    justify-content: center; /* Centers horizontally */
                    align-items: center; /* Centers vertically */
                    height: 100vh; /* Full viewport height */
                }

                div {
                    width: 50%;
                }
                
            </style>
            <body><div>
                <h1>404: Not Found</h1>
                <textarea style="width:100%; font-family: Monospace; font-size:10px; border:0;" rows="30" disabled>
                Try:
                    /api/chat
                             |- /login                              ---> Enables internal cookie.
                             |- /me                                 ---> Returns the user id given the user cookie.
                             |- /conversation                       ---> (GET) Lists conversations a user is in. (POST) Starts a conversation.
                                             |- /{convo_id}/peer    ---> Gets the jumpseller_id of the peer.
                                             |- /{convo_id}/latest  ---> Gets the latest message.
                                             |- /{convo_id}/recent  ---> Gets the 32 most recent messages.
                                             |- /{convo_id}/product ---> Gets the product associated with the conversation.
                                             |- /{convo_id}/message ---> Posts a new message into the chat.
                             |- /message/{msg_id}                   ---> Gets the message with ID 'msg_id'.
                             |- /user/{js_id}                       ---> Gets the profile of user with id 'js_id'.
                             |- /product                            ---> Posts a new product into the database.
                             |- /product/{prod_id}                  ---> Gets the product with id 'prod_id'.
                </textarea>
            </div></body>        
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

#[allow(dead_code)]
trait LogShort<E>: EasyLog<E>
where
    E: std::fmt::Display,
    Self: Sized,
{
    /// Logs as an Error
    fn e(self) -> Self {
        self.log(|e| log::error!("{e}"))
    }

    /// Logs as a Warning
    fn w(self) -> Self {
        self.log(|e| log::warn!("{e}"))
    }

    /// Logs as an Info
    fn i(self) -> Self {
        self.log(|e| log::info!("{e}"))
    }
}

impl<T, E> LogShort<E> for T
where
    T: EasyLog<E>,
    E: std::fmt::Display,
{
}

#[derive(Debug, Serialize, Deserialize)]
struct Credential {
    id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct MaybeCredential {
    id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthService {
    auth_service_user_id: Option<i64>,
}

#[derive(Debug, thiserror::Error)]
#[error("The current authentication state does not allow you to access this content.")]
struct ProductionAuthMissing;

impl ResponseError for ProductionAuthMissing {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::UNAUTHORIZED
    }
}

#[get("/login")]
async fn login(
    db: Data<RwLock<SQLiteDB>>,
    js: Data<jumpseller::Client>,
    prod: Data<IsProd>,
    auth: Query<AuthService>,
    user: Query<MaybeCredential>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let user_id = if let Some(user_id) = auth.auth_service_user_id {
        jumpseller_update_user(&db, &js, user_id).await?;
        user_id
    } else {
        if prod.is_prod() {
            return Err(ProductionAuthMissing.into());
        }
        let Some(user_id) = user.id else {
            return Err(ProductionAuthMissing.into());
        };
        jumpseller_update_user(&db, &js, user_id).await?;
        user_id
    };

    // Attach identity
    Identity::login(&req.extensions(), format!("{user_id}"))?;

    Ok(HttpResponse::Ok())
}

#[get("/me")]
async fn me(
    user: Identity,
    prod: Data<IsProd>,
    auth: Query<AuthService>,
) -> Result<impl Responder> {
    let id = parse_cookie(user.id()?)?;

    if prod.is_prod()
        && let Some(authid) = auth.auth_service_user_id
        && authid != id
    {
        return Err(ProductionAuthMissing.into());
    }

    if prod.is_prod() && auth.auth_service_user_id.is_none() {
        return Err(ProductionAuthMissing.into());
    }

    Ok(Json(Credential { id }))
}

// FIXME: usr_id needs be usr_token
#[get("/conversation")]
async fn get_conversations(
    user: Identity,
    db: Data<RwLock<SQLiteDB>>,
    auth: Query<AuthService>,
    prod: Data<IsProd>,
) -> Result<impl Responder> {
    // SAFETY: No need to refetch info, it is about ourselves.
    let user_id = parse_cookie(user.id()?)?;

    if prod.is_prod()
        && let Some(authid) = auth.auth_service_user_id
        && authid != user_id
    {
        return Err(ProductionAuthMissing.into());
    }

    if prod.is_prod() && auth.auth_service_user_id.is_none() {
        return Err(ProductionAuthMissing.into());
    }

    let user_id = UserId(user_id);

    let res = db
        .read()
        .await
        .get_conversations(&user_id)
        .await
        .map(Json)
        .w()?;
    Ok(res)
}

// FIXME: usr_id needs be usr_token
#[get("/conversation/{convo_id}/peer")]
async fn get_peer(
    data: Data<RwLock<SQLiteDB>>,
    user: Identity,
    convo_id: Path<i64>,
    auth: Query<AuthService>,
    prod: Data<IsProd>,
) -> Result<impl Responder> {
    #[derive(Serialize)]
    struct UserIdWrapper {
        id: i64,
    }

    let user_id = user.id().map(parse_cookie)??;

    if prod.is_prod()
        && let Some(authid) = auth.auth_service_user_id
        && authid != user_id
    {
        return Err(ProductionAuthMissing.into());
    }

    if prod.is_prod() && auth.auth_service_user_id.is_none() {
        return Err(ProductionAuthMissing.into());
    }

    let user_id = UserId(user_id);

    let convo_id = ConversationId(*convo_id);
    data.read()
        .await
        .belongs_to_conversation(&user_id, &convo_id)
        .await
        .w()?;
    let peer_id = data.read().await.get_peer(&user_id, &convo_id).await?;
    let profile = data.read().await.get_user_profile(&peer_id).await?;
    // SAFETY: no need to update the peer, as we are only getting their id

    let profile = UserIdWrapper { id: profile.id().0 };

    Ok(Json(profile))
}

#[get("/user/{user_id}")]
async fn get_user_profile(
    db: Data<RwLock<SQLiteDB>>,
    user_id: Path<i64>,
    js: Data<jumpseller::Client>,
) -> Result<impl Responder> {
    let user_id = UserId(*user_id);
    jumpseller_update_user(&db, &js, user_id.0).await?;
    let res = db
        .read()
        .await
        .get_user_profile(&user_id)
        .await
        .map(Json)
        .w()?;
    Ok(res)
}

#[derive(Debug, Serialize, Deserialize)]
struct MessageContent {
    sender_jsid: i64,
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
    fn new(sender_jsid: i64, msg: Message) -> Self {
        Self { sender_jsid, msg }
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
    auth: Query<AuthService>,
    prod: Data<IsProd>,
) -> Result<impl Responder> {
    let user_id = user.id().map(parse_cookie)?.map(UserId)?;
    if prod.is_prod()
        && let Some(authid) = auth.auth_service_user_id
        && authid != user_id.0
    {
        return Err(ProductionAuthMissing.into());
    }
    if prod.is_prod() && auth.auth_service_user_id.is_none() {
        return Err(ProductionAuthMissing.into());
    }

    let msg_id = MessageId(*msg_id);
    let convo_id = data
        .read()
        .await
        .get_conversation_from_message(&msg_id)
        .await
        .w()?;
    data.read()
        .await
        .belongs_to_conversation(&user_id, &convo_id)
        .await
        .log(|e| warn!("{e}"))?;
    let (sender_id, msg, prev_id) = data.read().await.get_message(&msg_id).await.w()?;
    let msg = MessageContent::new(sender_id.0, msg);
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
struct ConversationForm {
    their_userid: i64,
    product_jumpseller_id: i64,
}

#[derive(Debug, thiserror::Error)]
#[error("Parsing error: {0}")]
struct CookieParseError(#[from] ParseIntError);

fn parse_cookie<I: AsRef<str>>(identity: I) -> Result<i64, CookieParseError> {
    identity.as_ref().parse::<i64>().map_err(CookieParseError)
}

impl ResponseError for CookieParseError {}

#[post("/conversation")]
async fn start_conversation(
    utils: Data<BackendInfoUpdater>,
    data: Data<RwLock<SQLiteDB>>,
    jumpseller: Data<jumpseller::Client>,
    user: Identity,
    form: Form<ConversationForm>,
) -> Result<impl Responder> {
    #[derive(Serialize)]
    struct ConversationIdWrapper {
        id: i64,
    }
    let user_id = user.id().map(parse_cookie)?.map(UserId)?;
    let their_id = form.their_userid;

    jumpseller_update_user(&data, &jumpseller, user_id.0).await?;
    jumpseller_update_user(&data, &jumpseller, their_id).await?;
    jumpseller_update_product(
        &data,
        &jumpseller,
        &UserId(their_id),
        form.product_jumpseller_id,
    )
    .await?;

    let their_id = UserId(their_id);

    data.read()
        .await
        .belongs_to_seller(&their_id, &form.product_jumpseller_id.into())
        .await
        .w()?;
    let res = data
        .write()
        .await
        .start_conversation(&user_id, &their_id, &form.product_jumpseller_id.into())
        .await
        .w()?;

    // Don't divulge for now.
    let callback = utils.new_convo(&*data.read().await, &res, &user_id).await?;

    match callback.await.map_err(ErrorInternalServerError)? {
        crate::F2BResponse::Ok => {}
        crate::F2BResponse::GoogleCloud(error) => {
            log::error!("Failed to publish message: {error}.");
        }
        crate::F2BResponse::Unrecoverable(error) => {
            log::error!("Failed to publish message: {error}.");
        }
    }

    let res = ConversationIdWrapper { id: res.0 };

    Ok(Json(res))
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
struct MessageForm {
    message: String,
}

#[post("/conversation/{convo_id}/message")]
async fn post_msg(
    utils: Data<BackendInfoUpdater>,
    data: Data<RwLock<SQLiteDB>>,
    user: Identity,
    conversation: Path<i64>,
    form: Form<MessageForm>,
) -> Result<impl Responder> {
    #[derive(Serialize)]
    struct MessageIdWrapper {
        id: i64,
    }
    let user_id = user.id().map(parse_cookie)?.map(UserId)?;
    let convo_id = ConversationId(conversation.into_inner());
    data.read()
        .await
        .belongs_to_conversation(&user_id, &convo_id)
        .await
        .w()?;
    let msg = Message::from(form.into_inner().message.as_str());
    let res = data
        .write()
        .await
        .post_msg(msg, &user_id, &convo_id)
        .await
        .w()?;

    // Don't divulge for now.
    let callback = utils
        .new_message(&*data.read().await, &res, &convo_id, false)
        .await?;

    match callback.await.map_err(ErrorInternalServerError)? {
        crate::F2BResponse::Ok => {}
        crate::F2BResponse::GoogleCloud(error) => {
            log::error!("Failed to publish message: {error}.");
        }
        crate::F2BResponse::Unrecoverable(error) => {
            log::error!("Failed to publish message: {error}.");
        }
    }

    let res = MessageIdWrapper { id: res.0 };

    Ok(Json(res))
}

#[get("/conversation/{convo_id}/latest")]
async fn get_latest_message(
    data: Data<RwLock<SQLiteDB>>,
    user: Identity,
    convo_id: Path<i64>,
    auth: Query<AuthService>,
    prod: Data<IsProd>,
) -> Result<impl Responder> {
    #[derive(Serialize)]
    struct MaybeMsgIdWrapper {
        id: Option<i64>,
    }
    let user_id = user.id().map(parse_cookie)?.map(UserId)?;
    if prod.is_prod()
        && let Some(authid) = auth.auth_service_user_id
        && authid != user_id.0
    {
        return Err(ProductionAuthMissing.into());
    }
    if prod.is_prod() && auth.auth_service_user_id.is_none() {
        return Err(ProductionAuthMissing.into());
    }

    let convo_id = ConversationId(*convo_id);
    data.read()
        .await
        .belongs_to_conversation(&user_id, &convo_id)
        .await
        .w()?;
    let res = data.read().await.get_latest_message(&convo_id).await.w()?;

    let res = MaybeMsgIdWrapper {
        id: res.map(|x| x.0),
    };

    Ok(Json(res))
}

#[get("/conversation/{convo_id}/recent")]
async fn get_most_recent_messages(
    data: Data<RwLock<SQLiteDB>>,
    user: Identity,
    convo_id: Path<i64>,
    auth: Query<AuthService>,
    prod: Data<IsProd>,
) -> Result<impl Responder> {
    let user_id = user.id().map(parse_cookie)?.map(UserId)?;
    if prod.is_prod()
        && let Some(authid) = auth.auth_service_user_id
        && authid != user_id.0
    {
        return Err(ProductionAuthMissing.into());
    }
    if prod.is_prod() && auth.auth_service_user_id.is_none() {
        return Err(ProductionAuthMissing.into());
    }

    let convo_id = ConversationId(*convo_id);
    data.read()
        .await
        .belongs_to_conversation(&user_id, &convo_id)
        .await
        .w()?;
    let (messages, prev_id) = data
        .read()
        .await
        .get_most_recent_messages(&convo_id)
        .await
        .w()?;
    let mut msgs = Vec::new();
    for (sender_id, msg) in messages {
        msgs.push(MessageContent::new(sender_id.0, msg));
    }
    Ok(Json(MessageFormat::many(msgs, prev_id)))
}

#[get("/product/{prod_id}")]
async fn get_product(
    data: Data<RwLock<SQLiteDB>>,
    jumpseller: Data<jumpseller::Client>,
    prod_id: Path<i64>,
) -> Result<impl Responder> {
    let seller_id = data
        .read()
        .await
        .get_product(&ProductId(*prod_id))
        .await?
        .seller_id;
    jumpseller_update_product(&data, &jumpseller, &seller_id, *prod_id).await?;
    let prod = data.read().await.get_product(&ProductId(*prod_id)).await?;
    Ok(Json(prod))
}

#[get("/conversation/{convo_id}/product")]
async fn get_product_in_conversation(
    data: Data<RwLock<SQLiteDB>>,
    convo_id: Path<i64>,
    user: Identity,
    auth: Query<AuthService>,
    prod: Data<IsProd>,
) -> Result<impl Responder> {
    #[derive(Serialize)]
    struct ProductIdWrapper {
        id: i64,
    }
    let user_id = user.id().map(parse_cookie)?.map(UserId)?;
    if prod.is_prod()
        && let Some(authid) = auth.auth_service_user_id
        && authid != user_id.0
    {
        return Err(ProductionAuthMissing.into());
    }
    if prod.is_prod() && auth.auth_service_user_id.is_none() {
        return Err(ProductionAuthMissing.into());
    }

    data.read()
        .await
        .belongs_to_conversation(&user_id, &ConversationId(*convo_id))
        .await
        .w()?;
    let prod = data
        .read()
        .await
        .get_product_id_from_conversation_id(&ConversationId(*convo_id))
        .await
        .w()?;

    let prod = ProductIdWrapper { id: prod.0 };

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
    Ok(data
        .write()
        .await
        .add_product(&product)
        .await
        .w()
        .map(Json)?)
}
