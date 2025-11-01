use crate::database::{Database, sqlite::*};
use actix_web::{
    Responder, Result, get, post,
    web::{Data, Form, Json, Path},
};
use tokio::sync::RwLock;

pub fn create_services() -> actix_web::Scope {
    actix_web::web::scope("/api/chat")
        .service(get_conversations)
        .service(get_peer)
        .service(get_user_profile)
        .service(get_message)
        .service(get_latest_message)
        .service(add_user)
}

#[get("/{usr_id}/conversation")]
async fn get_conversations(
    usr_id: Path<i64>,
    data: Data<RwLock<SQLiteDB>>,
) -> Result<impl Responder> {
    let usr_id = UserId(usr_id.clone());
    let records = data.read().await.get_conversations(&usr_id).await;
    match records {
        Ok(i) => Ok(Json(i)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}

#[get("/{usr_id}/conversation/{convo_id}/peer")]
async fn get_peer(
    data: Data<RwLock<SQLiteDB>>,
    usr_id: Path<i64>,
    convo_id: Path<i64>,
) -> Result<impl Responder> {
    let usr_id = UserId(usr_id.clone());
    let convo_id = ConversationId(convo_id.clone());
    let record = data.read().await.get_peer(&usr_id, &convo_id).await;
    match record {
        Ok(peer) => Ok(Json(peer)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}

#[get("/{usr_id}/user")]
async fn get_user_profile(
    data: Data<RwLock<SQLiteDB>>,
    usr_id: Path<i64>,
) -> Result<impl Responder> {
    let usr_id = UserId(usr_id.clone());
    let record = data.read().await.get_user_profile(&usr_id).await;
    match record {
        Ok(profile) => Ok(Json(profile)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}

#[get("/{usr_id}/message/{msg_id}")]
async fn get_message(
    data: Data<RwLock<SQLiteDB>>,
    usr_id: Path<i64>,
    msg_id: Path<i64>,
) -> Result<impl Responder> {
    let usr_id = UserId(usr_id.clone());
    let msg_id = MessageId(msg_id.clone());
    let record = data.read().await.get_message(&msg_id).await;
    match record {
        Ok(message) => Ok(Json(message)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}

#[post("/user")]
async fn add_user(
    data: Data<RwLock<SQLiteDB>>,
    user_profile: Form<UserProfile>,
) -> Result<impl Responder> {
    let user_profile = user_profile.0;
    let result = data.write().await.add_user(&user_profile).await;
    match result {
        Ok(id) => Ok(Json(id)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}

// #[post("/{usr_id}/conversation")]
// async fn start_conversation(
//     data: Data<SQLiteDB>,
//     my_id: Path<i64>,
//     their_id: Json<UserId>,
// ) -> Result<impl Responder> {
//     todo!()
// }

// #[post("/{usr_id}/conversation/{convo_id}/message")]
// async fn post_msg(
//     data: Data<SQLiteDB>,
//     my_id: Path<i64>,
//     conversation: Path<i64>,
//     msg: Json<String>,
// ) -> Result<impl Responder> {
//     todo!()
// }

#[get("/{usr_id}/conversation/{convo_id}/latest")]
async fn get_latest_message(
    data: Data<RwLock<SQLiteDB>>,
    usr_id: Path<i64>,
    convo_id: Path<i64>,
) -> Result<impl Responder> {
    let usr_id = UserId(usr_id.clone());
    let convo_id = ConversationId(convo_id.clone());
    let record = data.read().await.get_latest_message(&convo_id).await;
    match record {
        Ok(message) => Ok(Json(message)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}
