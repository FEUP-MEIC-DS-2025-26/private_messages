use crate::database::{Database, sqlite::*};
use actix_web::{
    Responder, Result, get, post,
    web::{Data, Json, Path},
};

#[get("/api/chat/{usr_id}/conversation")]
async fn get_conversations(usr_id: Path<i64>, data: Data<SQLiteDB>) -> Result<impl Responder> {
    let usr_id = UserId(usr_id.clone());
    let records = data.get_conversations(&usr_id).await;
    match records {
        Ok(i) => Ok(Json(i)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}

#[get("/api/chat/{usr_id}/conversation/{convo_id}/peer")]
async fn get_peer(
    data: Data<SQLiteDB>,
    usr_id: Path<i64>,
    convo_id: Path<i64>,
) -> Result<impl Responder> {
    let usr_id = UserId(usr_id.clone());
    let convo_id = ConversationId(convo_id.clone());
    let record = data.get_peer(&usr_id, &convo_id).await;
    match record {
        Ok(peer) => Ok(Json(peer)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}

#[get("/api/chat/{usr_id}/user")]
async fn get_user_profile(data: Data<SQLiteDB>, usr_id: Path<i64>) -> Result<impl Responder> {
    let usr_id = UserId(usr_id.clone());
    let record = data.get_user_profile(&usr_id).await;
    match record {
        Ok(profile) => Ok(Json(profile)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}

#[get("/api/chat/{usr_id}/message/{msg_id}")]
async fn get_message(
    data: Data<SQLiteDB>,
    usr_id: Path<i64>,
    msg_id: Path<i64>,
) -> Result<impl Responder> {
    let usr_id = UserId(usr_id.clone());
    let msg_id = MessageId(msg_id.clone());
    let record = data.get_message(&msg_id).await;
    match record {
        Ok(message) => Ok(Json(message)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}

// #[post("/api/chat/{usr_id}/user")]
// async fn add_user(
//     data: Data<SQLiteDB>,
//     usr_id: Path<i64>,
//     user_profile: Json<UserProfile>,
// ) -> Result<impl Responder> {
//     todo!()
// }

// #[post("/api/chat/{usr_id}/conversation")]
// async fn start_conversation(
//     data: Data<SQLiteDB>,
//     my_id: Path<i64>,
//     their_id: Json<UserId>,
// ) -> Result<impl Responder> {
//     todo!()
// }

// #[post("/api/chat/{usr_id}/conversation/{convo_id}/message")]
// async fn post_msg(
//     data: Data<SQLiteDB>,
//     my_id: Path<i64>,
//     conversation: Path<i64>,
//     msg: Json<String>,
// ) -> Result<impl Responder> {
//     todo!()
// }

#[get("/api/chat/{usr_id}/conversation/{convo_id}/latest")]
async fn get_latest_message(
    data: Data<SQLiteDB>,
    usr_id: Path<i64>,
    convo_id: Path<i64>,
) -> Result<impl Responder> {
    let usr_id = UserId(usr_id.clone());
    let convo_id = ConversationId(convo_id.clone());
    let record = data.get_latest_message(&convo_id).await;
    match record {
        Ok(message) => Ok(Json(message)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}
