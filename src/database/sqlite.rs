use crate::database::Database;
use actix_web::{ResponseError, http::StatusCode};
use serde;
use sqlx::{Pool, Sqlite, migrate::MigrateDatabase, sqlite::SqlitePoolOptions};

pub struct SQLiteDB {
    pool: Pool<Sqlite>,
}

impl SQLiteDB {
    pub async fn new(url: &str, populate: bool) -> anyhow::Result<Self> {
        if populate {
            Sqlite::drop_database(url).await?;
        }
        
        if !Sqlite::database_exists(url).await? {
            Sqlite::create_database(url).await?;
        }
        let pool = SqlitePoolOptions::new().connect_lazy(url)?;
        let mut db = SQLiteDB { pool };
        db.set_schema().await?;
        if populate {
            sqlx::query_file!("src/database/populate.sql").execute(&db.pool).await?;
        }
        Ok(db)
    }

    async fn set_schema(&mut self) -> anyhow::Result<()> {
        sqlx::query_file!("src/database/schema.sql")
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[derive(Debug, sqlx::Type, PartialEq, Copy, Clone, serde::Serialize, serde::Deserialize)]
#[sqlx(transparent)]
pub struct UserId(pub i64);

#[derive(Debug, sqlx::Type, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserProfile {
    username: String,
    name: String,
}

impl UserProfile {
    pub fn new(username: String, name: String) -> Self {
        Self { username, name }
    }

    pub fn username(&self) -> String {
        self.username.clone()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Debug, sqlx::Type, PartialEq, Copy, Clone, serde::Serialize, serde::Deserialize)]
#[sqlx(transparent)]
pub struct ConversationId(pub i64);

#[derive(Debug, sqlx::Type, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[sqlx(transparent)]
pub struct Message(pub String);

#[derive(Debug, sqlx::Type, PartialEq, Copy, Clone, serde::Serialize, serde::Deserialize)]
#[sqlx(transparent)]
pub struct MessageId(pub i64);

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("Attempted to access something without needed priviledges")]
    PermissionDenied,
}

impl ResponseError for DbError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match &self {
            DbError::Db(error) => match error {
                sqlx::Error::Configuration(_) => StatusCode::INTERNAL_SERVER_ERROR,
                sqlx::Error::InvalidArgument(_) => StatusCode::BAD_REQUEST,
                sqlx::Error::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
                sqlx::Error::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
                sqlx::Error::Tls(_) => StatusCode::INTERNAL_SERVER_ERROR,
                sqlx::Error::Protocol(_) => StatusCode::INTERNAL_SERVER_ERROR,
                sqlx::Error::RowNotFound => StatusCode::NO_CONTENT,
                sqlx::Error::TypeNotFound { type_name: _ } => StatusCode::INTERNAL_SERVER_ERROR,
                sqlx::Error::ColumnIndexOutOfBounds { index: _, len: _ } => {
                    StatusCode::INTERNAL_SERVER_ERROR
                }
                sqlx::Error::ColumnNotFound(_) => StatusCode::INTERNAL_SERVER_ERROR,
                sqlx::Error::ColumnDecode {
                    index: _,
                    source: _,
                } => StatusCode::INTERNAL_SERVER_ERROR,
                sqlx::Error::Encode(_) => StatusCode::INTERNAL_SERVER_ERROR,
                sqlx::Error::Decode(_) => StatusCode::INTERNAL_SERVER_ERROR,
                sqlx::Error::AnyDriverError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                sqlx::Error::PoolTimedOut => StatusCode::INTERNAL_SERVER_ERROR,
                sqlx::Error::PoolClosed => StatusCode::INTERNAL_SERVER_ERROR,
                sqlx::Error::WorkerCrashed => StatusCode::TOO_MANY_REQUESTS,
                sqlx::Error::Migrate(_) => StatusCode::INTERNAL_SERVER_ERROR,
                sqlx::Error::InvalidSavePointStatement => StatusCode::BAD_REQUEST,
                sqlx::Error::BeginFailed => StatusCode::INTERNAL_SERVER_ERROR,
                _ => StatusCode::IM_A_TEAPOT,
            },
            DbError::PermissionDenied => StatusCode::FORBIDDEN,
        }
    }
}
impl Database for SQLiteDB {
    type Error = DbError;

    type UserId = UserId;

    type UserProfile = UserProfile;

    type ConversationId = ConversationId;

    type MessageId = MessageId;

    type Message = Message;

    type Querier<'a> = &'a Pool<Sqlite>;

    async fn get_conversations(
        &self,
        my_id: &Self::UserId,
    ) -> Result<Vec<Self::ConversationId>, Self::Error> {
        let record = sqlx::query!(
            r#"
            SELECT id as "id!"
            FROM conversation 
            WHERE sender_id = ? OR receiver_id = ?;
        "#,
            my_id,
            my_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(record.iter().map(|r| ConversationId(r.id)).collect())
    }

    async fn get_peer(
        &self,
        my_id: &Self::UserId,
        conversation: &Self::ConversationId,
    ) -> Result<Self::UserId, Self::Error> {
        let record = sqlx::query!(
            r#"
            SELECT id as "id!"
            FROM conversation
            WHERE id = ? AND (sender_id = ? OR receiver_id = ?)
        "#,
            conversation,
            my_id,
            my_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(UserId(record.id))
    }

    async fn get_user_profile(
        &self,
        their_id: &Self::UserId,
    ) -> Result<Self::UserProfile, Self::Error> {
        sqlx::query_as!(
            UserProfile,
            r#"
            SELECT username as "username!", name as "name!"
            FROM user
            WHERE id = ?
        "#,
            their_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn get_message(
        &self,
        message: &Self::MessageId,
    ) -> Result<(Self::UserId, Self::Message, Option<Self::MessageId>), Self::Error> {
        let result = sqlx::query!(
            r#"
            SELECT sender_id as "sender_id!", content as "content!", previous_message_id
            FROM message
            WHERE id = ?
        "#,
            message
        )
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(res) => Ok((
                UserId(res.sender_id),
                Message(String::from_utf8_lossy(res.content.as_slice()).to_string()),
                res.previous_message_id.map(|x| MessageId(x)),
            )),
            Err(e) => Err(e.into()),
        }
    }

    async fn get_querier<'a>(&'a self) -> Result<Self::Querier<'a>, Self::Error> {
        Ok(&self.pool)
    }

    async fn add_user(&mut self, profile: &Self::UserProfile) -> Result<Self::UserId, Self::Error> {
        let mut transaction = self.pool.begin().await?;

        let record = sqlx::query!(
            r#"
            SELECT id as "id!"
            FROM user
            WHERE username = ?;
        "#,
            profile.username
        )
        .fetch_optional(&mut *transaction)
        .await?;

        if let Some(user) = record {
            return Ok(UserId(user.id));
        }

        let record = sqlx::query!(
            r#"
            INSERT INTO user (username, name)
            VALUES (?, ?)
            RETURNING id as "id!"
        "#,
            profile.username,
            profile.name
        )
        .fetch_one(&mut *transaction)
        .await;

        transaction.commit().await?;
        Ok(record.map(|i| UserId(i.id))?)
    }

    async fn start_conversation(
        &mut self,
        my_id: &Self::UserId,
        their_id: &Self::UserId,
    ) -> Result<Self::ConversationId, Self::Error> {
        let mut transaction = self.pool.begin().await?;

        let record = sqlx::query!(
            r#"
            SELECT id as "id!"
            FROM conversation
            WHERE (sender_id = ? AND receiver_id = ?) OR (receiver_id = ? AND sender_id = ?);
        "#,
            my_id,
            their_id,
            my_id,
            their_id
        )
        .fetch_optional(&mut *transaction)
        .await?;

        if let Some(convo) = record {
            return Ok(ConversationId(convo.id));
        }

        let record = sqlx::query!(
            r#"
            INSERT INTO conversation (sender_id, receiver_id)
            VALUES (?, ?)
            RETURNING id as "id!"
        "#,
            my_id,
            their_id
        )
        .fetch_one(&mut *transaction)
        .await;

        transaction.commit().await?;
        Ok(record.map(|i| ConversationId(i.id))?)
    }

    async fn post_msg(
        &mut self,
        msg: Self::Message,
        my_id: &Self::UserId,
        conversation: &Self::ConversationId,
    ) -> Result<Self::MessageId, Self::Error> {
        let mut transaction = self.pool.begin().await?;

        let prev_id = sqlx::query!(
            r#"
            SELECT last_message_id
            FROM conversation
            WHERE id = ?;
        "#,
            conversation
        )
        .fetch_one(&mut *transaction)
        .await?
        .last_message_id;

        let msg_id = sqlx::query!(
            r#"
            INSERT INTO message (content, sender_id, conversation_id, previous_message_id)
            VALUES (?, ?, ?, ?)
            RETURNING id as "id!"
        "#,
            msg,
            my_id,
            conversation,
            prev_id
        )
        .fetch_one(&mut *transaction)
        .await?
        .id;

        sqlx::query!(
            r#"
            UPDATE conversation
            SET last_message_id = ?
            WHERE id = ?;
        "#,
            msg_id,
            conversation
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
        Ok(MessageId(msg_id))
    }

    async fn get_latest_message(
        &self,
        conversation: &Self::ConversationId,
    ) -> Result<Option<Self::MessageId>, Self::Error> {
        let record = sqlx::query!(
            r#"
            SELECT last_message_id
            FROM conversation
            WHERE id = ?;
        "#,
            conversation
        )
        .fetch_one(&self.pool)
        .await;

        Ok(record.map(|i| i.last_message_id.map(|i| MessageId(i)))?)
    }

    async fn get_user_id_from_username(&self, username: &str) -> Result<Self::UserId, Self::Error> {
        let record = sqlx::query!(
            r#"
            SELECT id as "id!"
            FROM user
            WHERE user.username = ?;  
          "#,
            username
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(UserId(record.id))
    }

    async fn belongs_to_conversation(
        &self,
        id: &Self::UserId,
        conversation: &Self::ConversationId,
    ) -> Result<(), Self::Error> {
        let record = sqlx::query!(
            r#"
            SELECT EXISTS (
                SELECT conversation.id as "id!"
                FROM conversation
                WHERE conversation.id = ? AND (sender_id = ? OR receiver_id = ?)
            ) as is_there
          "#,
            conversation,
            id,
            id
        )
        .fetch_one(&self.pool)
        .await?;
        match record.is_there == 1 {
            true => Ok(()),
            false => Err(DbError::PermissionDenied),
        }
    }

    async fn get_conversation_from_message(
        &self,
        msg_id: &Self::MessageId,
    ) -> Result<Self::ConversationId, Self::Error> {
        let record = sqlx::query!(
            r#"
            SELECT conversation_id as "conversation_id!"
            FROM message
            WHERE id = ?
            "#, msg_id
        ).fetch_one(&self.pool).await?;
        Ok(ConversationId(record.conversation_id))
    }
}

#[cfg(test)]
mod test {
    use crate::database::Database;
    use crate::database::sqlite::*;

    #[tokio::test]
    async fn test_sqlite() -> anyhow::Result<()> {
        let alice = UserProfile {
            username: "alice_11".to_owned(),
            name: "Alice Arnold".to_owned(),
        };

        let bob = UserProfile {
            username: "bobert22".to_owned(),
            name: "Bob Bellows".to_owned(),
        };

        let mut db = SQLiteDB::new("sqlite::memory:", false).await?;

        let alice_id = db.add_user(&alice).await?;
        let bob_id = db.add_user(&bob).await?;
        assert_ne!(alice_id, bob_id);

        let alice_again = db.add_user(&alice).await?;
        assert_eq!(alice_id, alice_again);

        let convo_id = db.start_conversation(&alice_id, &bob_id).await?;
        let same_id = db.start_conversation(&bob_id, &alice_id).await?;

        assert_eq!(convo_id, same_id);

        let hello = Message("Hello Bob!".to_owned());
        let hello_id = db.post_msg(hello, &alice_id, &convo_id).await?;

        // Example queries
        let alice_bob_messages: Vec<Message> = {
            let querier = db.get_querier().await?;
            // Count all messages between Alice and Bob
            let msg_count = sqlx::query_scalar!(
                r#"
                SELECT COUNT(*) as "count!: i64"
                FROM message
                WHERE conversation_id = ?;
            "#,
                convo_id
            )
            .fetch_one(querier)
            .await?;
            let msg_count: i64 = msg_count.into();
            assert_eq!(msg_count, 1);
            // Make sure that the only message is the 'Hello Bob!' one.
            let msg_id = sqlx::query!(
                r#"
                SELECT id as "id!"
                FROM message
                WHERE conversation_id = ?;
            "#,
                convo_id
            )
            .fetch_one(querier)
            .await?;
            assert!(MessageId(msg_id.id) == hello_id);
            // Retrieve all messages between Alice and Bob
            let messages = sqlx::query!(
                r#"
                SELECT content as "content!"
                FROM message
                WHERE conversation_id = ?;
            "#,
                convo_id
            )
            .fetch_all(querier)
            .await?;
            messages
                .iter()
                .map(|m| Message(String::from_utf8_lossy(m.content.as_slice()).to_string()))
                .collect()
        };

        assert_eq!(alice_bob_messages, vec![Message("Hello Bob!".to_owned())]);

        Ok(())
    }

    #[tokio::test]
    async fn test_conversation_pointer() -> anyhow::Result<()> {
        // Preparation
        let alice = UserProfile {
            username: "alice_11".to_owned(),
            name: "Alice Arnold".to_owned(),
        };

        let bob = UserProfile {
            username: "bobert22".to_owned(),
            name: "Bob Bellows".to_owned(),
        };

        let mut db = SQLiteDB::new("sqlite::memory:", false).await?;

        let alice_id = db.add_user(&alice).await?;
        let bob_id = db.add_user(&bob).await?;
        assert_ne!(alice_id, bob_id);

        let alice_again = db.add_user(&alice).await?;
        assert_eq!(alice_id, alice_again);

        let convo_id = db.start_conversation(&alice_id, &bob_id).await?;

        // Test
        let last_msg = db.get_latest_message(&convo_id).await?;
        assert_eq!(last_msg, None);

        let hello = Message("Hello Bob!".to_owned());
        let first_hello_id = db.post_msg(hello, &alice_id, &convo_id).await?;

        let last_msg = db.get_latest_message(&convo_id).await?;
        assert_eq!(last_msg, Some(first_hello_id));

        let hello = Message("Hello Alice!".to_owned());
        let second_hello_id = db.post_msg(hello, &bob_id, &convo_id).await?;

        let last_msg = db.get_latest_message(&convo_id).await?.unwrap();
        assert_eq!(last_msg, second_hello_id);

        let (_, _, last_msg) = db.get_message(&last_msg).await?;
        assert_eq!(last_msg, Some(first_hello_id));
        Ok(())
    }
}
