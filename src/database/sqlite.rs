use crate::database::Database;
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Pool, Sqlite};

pub struct SQLiteDB {
    pool: Pool<Sqlite>,
}

impl SQLiteDB {
    pub async fn new(url: &str) -> anyhow::Result<Self> {
        if !Sqlite::database_exists(url).await? {
            Sqlite::create_database(url).await?;
        }
        let pool = SqlitePoolOptions::new().connect_lazy(url)?;
        let mut db = SQLiteDB { pool };
        db.set_schema().await?;
        Ok(db)
    }
    
    async fn set_schema(&mut self) -> anyhow::Result<()> {
        sqlx::query_file!("src/database/schema.sql").execute(&self.pool).await?;
        Ok(())
    }
}

#[derive(Debug, sqlx::Type, PartialEq, Copy, Clone)]
pub struct UserId {
    id: i64
}

#[derive(Debug, sqlx::Type, PartialEq, Clone)]
pub struct UserProfile {
    username: String,
    name: String,
}

#[derive(Debug, sqlx::Type, PartialEq, Copy, Clone)]
pub struct ConversationId {
    id: i64
}

#[derive(Debug, sqlx::Type, PartialEq, Clone)]
pub struct Message {
    content: String
}

#[derive(Debug, sqlx::Type, PartialEq, Copy, Clone)]
pub struct MessageId {
    id: i64
}

impl Database for SQLiteDB {
    type Error = sqlx::Error;

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
        sqlx::query_as!(ConversationId, r#"
            SELECT id as "id!"
            FROM conversation 
            WHERE sender_id = ? OR receiver_id = ?;
        "#, my_id.id, my_id.id).fetch_all(&self.pool).await
    }

    async fn get_peer(
        &self,
        my_id: &Self::UserId,
        conversation: &Self::ConversationId,
    ) -> Result<Self::UserId, Self::Error> {
        sqlx::query_as!(UserId, r#"
            SELECT id as "id!"
            FROM conversation
            WHERE id = ? AND (sender_id = ? OR receiver_id = ?)
        "#, conversation.id, my_id.id, my_id.id).fetch_one(&self.pool).await
    }

    async fn get_user_profile(
        &self,
        their_id: &Self::UserId,
    ) -> Result<Self::UserProfile, Self::Error> {
        sqlx::query_as!(UserProfile, r#"
            SELECT username as "username!", name as "name!"
            FROM user
            WHERE id = ?
        "#, their_id.id).fetch_one(&self.pool).await
    }

    async fn get_message(
        &self,
        message: &Self::MessageId
    ) -> Result<(Self::UserId, Self::Message, Option<Self::MessageId>), Self::Error> {
        let result = sqlx::query!(r#"
            SELECT sender_id as "sender_id!", content as "content!", previous_message_id
            FROM message
            WHERE id = ?
        "#, message.id).fetch_one(&self.pool).await;
        
        match result {
            Ok(res) => Ok((
                UserId {id: res.sender_id}, 
                Message {content: String::from_utf8_lossy(res.content.as_slice()).to_string()}, 
                res.previous_message_id.map(|x| MessageId {id: x})
            )),
            Err(e) => Err(e)
        }
    }

    async fn get_querier<'a>(&'a self) -> Result<Self::Querier<'a>, Self::Error> {
        Ok(&self.pool)
    }

    async fn add_user(&mut self, profile: &Self::UserProfile) -> Result<Self::UserId, Self::Error> {
        let mut transaction = self.pool.begin().await?;
        
        let existing_user = sqlx::query_as!(Self::UserId, r#"
            SELECT id as "id!"
            FROM user
            WHERE username = ?;
        "#, profile.username).fetch_optional(&mut *transaction).await?;
        
        if let Some(user) = existing_user {
            return Ok(user);
        }
        
        let record = sqlx::query!(r#"
            INSERT INTO user (username, name)
            VALUES (?, ?)
            RETURNING id as "id!"
        "#, profile.username, profile.name).fetch_one(&mut *transaction).await;
        
        transaction.commit().await?;
        record.map(|i| Self::UserId { id: i.id })
    }

    async fn start_conversation(
        &mut self,
        my_id: &Self::UserId,
        their_id: &Self::UserId,
    ) -> Result<Self::ConversationId, Self::Error> {
        let mut transaction = self.pool.begin().await?;
        
        let existing_convo = sqlx::query!(r#"
            SELECT id as "id!"
            FROM conversation
            WHERE (sender_id = ? AND receiver_id = ?) OR (receiver_id = ? AND sender_id = ?);
        "#, my_id.id, their_id.id, my_id.id, their_id.id).fetch_optional(&mut *transaction).await?;
        
        if let Some(record) = existing_convo {
            return Ok(Self::ConversationId { id: record.id });
        }
        
        let record = sqlx::query!(r#"
            INSERT INTO conversation (sender_id, receiver_id)
            VALUES (?, ?)
            RETURNING id as "id!"
        "#, my_id.id, their_id.id).fetch_one(&mut *transaction).await;
        
        transaction.commit().await?;
        record.map(|i| Self::ConversationId { id: i.id })
    }

    async fn post_msg(
        &mut self,
        msg: Self::Message,
        my_id: &Self::UserId,
        conversation: &Self::ConversationId,
    ) -> Result<Self::MessageId, Self::Error> {
        let mut transaction = self.pool.begin().await?;
        
        let prev_id = sqlx::query!(r#"
            SELECT last_message_id
            FROM conversation
            WHERE id = ?;
        "#, conversation.id).fetch_one(&mut *transaction).await?.last_message_id;
        
        let msg_id = sqlx::query!(r#"
            INSERT INTO message (content, sender_id, conversation_id, previous_message_id)
            VALUES (?, ?, ?, ?)
            RETURNING id as "id!"
        "#, msg.content, my_id.id, conversation.id, prev_id).fetch_one(&mut *transaction).await?.id;
        
        sqlx::query!(r#"
            UPDATE conversation
            SET last_message_id = ?
            WHERE id = ?;
        "#, msg_id, conversation.id).execute(&mut *transaction).await?;
        
        transaction.commit().await?;
        Ok(Self::MessageId { id: msg_id })
    }

    async fn get_latest_message(
        &self,
        conversation: &Self::ConversationId
    ) -> Result<Option<Self::MessageId>, Self::Error> {
        let record = sqlx::query!(r#"
            SELECT last_message_id
            FROM conversation
            WHERE id = ?;
        "#, conversation.id).fetch_one(&self.pool).await;
        
        record.map(|i| i.last_message_id.map(|i| Self::MessageId { id: i }))
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

        let mut db = SQLiteDB::new("sqlite::memory:").await?;

        let alice_id = db.add_user(&alice).await?;
        let bob_id = db.add_user(&bob).await?;
        assert_ne!(alice_id, bob_id);

        let alice_again = db.add_user(&alice).await?;
        assert_eq!(alice_id, alice_again);

        let convo_id = db.start_conversation(&alice_id, &bob_id).await?;
        let same_id = db.start_conversation(&bob_id, &alice_id).await?;

        assert_eq!(convo_id, same_id);

        let hello = Message { content: "Hello Bob!".to_owned() };
        let hello_id = db.post_msg(hello, &alice_id, &convo_id).await?;

        // Example queries
        let alice_bob_messages: Vec<Message> = {
            let querier = db.get_querier().await?;
            // Count all messages between Alice and Bob
            let msg_count = sqlx::query_scalar!(r#"
                SELECT COUNT(*) as "count!: i64"
                FROM message
                WHERE conversation_id = ?;
            "#, convo_id.id).fetch_one(querier).await?;
            let msg_count: i64 = msg_count.into();
            assert_eq!(msg_count, 1);
            // Make sure that the only message is the 'Hello Bob!' one.
            let msg_id = sqlx::query!(r#"
                SELECT id as "id!"
                FROM message
                WHERE conversation_id = ?;
            "#, convo_id.id).fetch_one(querier).await?;
            assert!(MessageId {id: msg_id.id} == hello_id);
            // Retrieve all messages between Alice and Bob
            let messages = sqlx::query!(r#"
                SELECT content as "content!"
                FROM message
                WHERE conversation_id = ?;
            "#, convo_id.id).fetch_all(querier).await?;
            messages.iter().map(|m| Message {content: String::from_utf8_lossy(m.content.as_slice()).to_string()}).collect()
        };

        assert_eq!(alice_bob_messages, vec![Message {content: "Hello Bob!".to_owned() }]);

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

        let mut db = SQLiteDB::new("sqlite::memory:").await?;

        let alice_id = db.add_user(&alice).await?;
        let bob_id = db.add_user(&bob).await?;
        assert_ne!(alice_id, bob_id);

        let alice_again = db.add_user(&alice).await?;
        assert_eq!(alice_id, alice_again);

        let convo_id = db.start_conversation(&alice_id, &bob_id).await?;
        
        // Test
        let last_msg = db.get_latest_message(&convo_id).await?;
        assert_eq!(last_msg, None);
        
        let hello = Message { content: "Hello Bob!".to_owned() };
        let first_hello_id = db.post_msg(hello, &alice_id, &convo_id).await?;
        
        let last_msg = db.get_latest_message(&convo_id).await?;
        assert_eq!(last_msg, Some(first_hello_id));
        
        let hello = Message { content: "Hello Alice!".to_owned() };
        let second_hello_id = db.post_msg(hello, &bob_id, &convo_id).await?;
        
        let last_msg = db.get_latest_message(&convo_id).await?.unwrap();
        assert_eq!(last_msg, second_hello_id);
        
        let (_, _, last_msg) = db.get_message(&last_msg).await?;
        assert_eq!(last_msg, Some(first_hello_id));
        Ok(())
    }
}
