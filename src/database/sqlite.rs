use std::ops::Deref;

use crate::database::{
    Database,
    crypto::{CryptData, CryptError, CryptoKey},
};
use actix_web::{ResponseError, http::StatusCode};
use chrono::{DateTime, Utc};
use rand::{SeedableRng, rngs::StdRng};
use serde;
use sqlx::{Pool, Sqlite, migrate::MigrateDatabase, sqlite::SqlitePoolOptions};

pub struct SQLiteDB {
    pool: Pool<Sqlite>,
    suite: CryptoKey,
    rng: StdRng,
}

impl SQLiteDB {
    pub async fn new(url: &str, suite: CryptoKey) -> anyhow::Result<Self> {
        if !Sqlite::database_exists(url).await? {
            Sqlite::create_database(url).await?;
        }
        let rng = StdRng::from_os_rng();
        let pool = SqlitePoolOptions::new().connect_lazy(url)?;
        let mut db = Self { pool, suite, rng };
        db.set_schema().await?;
        Ok(db)
    }

    pub async fn kiosk(suite: CryptoKey) -> anyhow::Result<Self> {
        let pool = SqlitePoolOptions::new().connect_lazy("sqlite::memory:")?;
        let rng = StdRng::from_os_rng();
        let mut db = Self { pool, suite, rng };
        db.set_schema().await?;
        for user in Self::kiosk_users() {
            db.add_user(&user).await?;
        }
        for product in Self::kiosk_products() {
            db.add_product(&product).await?;
        }

        for (my_id, their_id, prod_id) in Self::kiosk_conversations() {
            db.start_conversation(&my_id, &their_id, &prod_id).await?;
        }
        for (msg, sender, convo) in Self::kiosk_messages() {
            db.post_msg(msg, &sender, &convo).await?;
        }
        Ok(db)
    }

    fn kiosk_products() -> Vec<Product> {
        vec![
            Product {
                name: "Orange".to_owned(),
                seller_id: UserId(2),
                jumpseller_id: 9347673,
            },
            Product {
                name: "Orange Cake".to_owned(),
                seller_id: UserId(1),
                jumpseller_id: 9347699,
            },
        ]
    }

    fn kiosk_users() -> Vec<UserProfile> {
        vec![
            UserProfile::new_clone("john", "John Doe"),
            UserProfile::new_clone("jane", "Jane Doe"),
            UserProfile::new_clone("fred", "Fred Nerk"),
        ]
    }

    fn kiosk_conversations() -> Vec<(UserId, UserId, ProductId)> {
        vec![
            (UserId(1), UserId(2), ProductId(1)),
            (UserId(1), UserId(3), ProductId(2)),
        ]
    }

    fn kiosk_messages() -> Vec<(Message, UserId, ConversationId)> {
        vec![
            (
                Message::from("Hi Jane! I would like to buy a few oranges, are they fresh?"),
                UserId(1),
                ConversationId(1),
            ),
            (
                Message::from("Yes John! I just collected them this morning!"),
                UserId(2),
                ConversationId(1),
            ),
            (
                Message::from("Thank you for the clarification!"),
                UserId(1),
                ConversationId(1),
            ),
            (
                Message::from("Hi John! Is your orange cake made from fresh oranges?"),
                UserId(3),
                ConversationId(2),
            ),
            (
                Message::from("Yes Fred! I bought them today from Jane!"),
                UserId(1),
                ConversationId(2),
            ),
            (
                Message::from("Amazing! That makes me relieved, thank you!"),
                UserId(3),
                ConversationId(2),
            ),
        ]
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
    #[allow(dead_code)]
    pub fn new(username: String, name: String) -> Self {
        Self { username, name }
    }

    pub fn new_clone(username: &str, name: &str) -> Self {
        Self {
            username: username.to_owned(),
            name: name.to_owned(),
        }
    }

    pub fn username(&self) -> String {
        self.username.clone()
    }

    #[allow(dead_code)]
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Debug, sqlx::Type, PartialEq, Copy, Clone, serde::Serialize, serde::Deserialize)]
#[sqlx(transparent)]
pub struct ConversationId(pub i64);

#[derive(Debug, sqlx::Type, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    contents: String,
    timestamp: DateTime<Utc>,
}

impl Message {
    pub fn new(contents: String, timestamp: DateTime<Utc>) -> Self {
        Self {
            contents,
            timestamp,
        }
    }

    pub fn message(&self) -> &str {
        &self.contents
    }

    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }
}

impl From<&str> for Message {
    fn from(value: &str) -> Self {
        let owned = value.to_owned();
        let timestamp = Utc::now();
        Self::new(owned, timestamp)
    }
}

#[derive(Debug, sqlx::Type, PartialEq, Copy, Clone, serde::Serialize, serde::Deserialize)]
#[sqlx(transparent)]
pub struct MessageId(pub i64);

#[derive(Debug, sqlx::Type, PartialEq, Copy, Clone, serde::Serialize, serde::Deserialize)]
#[sqlx(transparent)]
pub struct ProductId(pub i64);

#[derive(Debug, sqlx::Type, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Product {
    name: String,
    seller_id: UserId,
    jumpseller_id: i64,
}

impl Product {
    pub fn new(name: String, seller_id: UserId, jumpseller_id: i64) -> Self {
        Self {
            name,
            seller_id,
            jumpseller_id,
        }
    }
    pub fn product_info(&self) -> i64 {
        self.jumpseller_id
    }
}

impl From<i64> for UserId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<i64> for ConversationId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<i64> for ProductId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<i64> for MessageId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("Attempted to access something without needed priviledges")]
    PermissionDenied,
    #[error("Wrong Salt Size. Must be exactly 12 bytes.")]
    SaltWrongSize,
    #[error(transparent)]
    Crypto(#[from] CryptError),
}

#[allow(dead_code)]
pub struct Querier<'a> {
    q: &'a Pool<Sqlite>,
    key: &'a CryptoKey,
    rng: &'a StdRng,
}

impl<'a> Deref for Querier<'a> {
    type Target = Pool<Sqlite>;

    fn deref(&self) -> &Self::Target {
        &self.q
    }
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
            DbError::SaltWrongSize => StatusCode::INTERNAL_SERVER_ERROR,
            DbError::Crypto(e) => e.status_code(),
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

    type ProductId = ProductId;

    type Product = Product;

    type Querier<'a> = Querier<'a>;

    async fn get_conversations(
        &self,
        my_id: &Self::UserId,
    ) -> Result<Vec<Self::ConversationId>, Self::Error> {
        let record = sqlx::query!(
            r#"
            SELECT id as "id!"
            FROM conversation 
            WHERE client_id = ? OR seller_id = ?;
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
            SELECT client_id as "client_id!", seller_id as "seller_id!"
            FROM conversation
            WHERE id = ? AND (client_id = ? OR seller_id = ?)
        "#,
            conversation,
            my_id,
            my_id
        )
        .fetch_one(&self.pool)
        .await?;

        if record.client_id == my_id.0 {
            Ok(UserId(record.seller_id))
        } else if record.seller_id == my_id.0 {
            Ok(UserId(record.client_id))
        } else {
            Err(DbError::PermissionDenied)
        }
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
            SELECT sender_id as "sender_id!", content as "content!", timestamp as "timestamp!", salt as "salt!", previous_message_id
            FROM message
            WHERE id = ?
        "#,
            message
        )
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(res) => {
                let contents = CryptData::from(res.content).decrypt(
                    &self.suite,
                    &res.salt.try_into().map_err(|_| DbError::SaltWrongSize)?,
                )?;
                Ok((
                    UserId(res.sender_id),
                    Message::new(contents, res.timestamp.and_utc()),
                    res.previous_message_id.map(MessageId),
                ))
            }
            Err(e) => Err(e.into()),
        }
    }

    async fn get_most_recent_messages(
        &self,
        conversation_id: &Self::ConversationId,
    ) -> Result<(Vec<(Self::UserId, Self::Message)>, Option<Self::MessageId>), Self::Error> {
        let result = sqlx::query!(r#"
            WITH id_asc as (
                SELECT id, sender_id, content, salt, timestamp, previous_message_id
                FROM message
                WHERE conversation_id = ?
                ORDER BY id desc
                LIMIT 32
            )
            SELECT sender_id as "sender_id!", content as "content!", salt as "salt!", timestamp as "timestamp!", previous_message_id FROM id_asc ORDER BY id
        "#,
            conversation_id
        ).fetch_all(&self.pool)
        .await;

        match result {
            Ok(res) => Ok((
                res.iter()
                    .map(|record| -> Result<(Self::UserId, Self::Message), DbError> {
                        let contents = CryptData::from(record.content.clone()).decrypt(
                            &self.suite,
                            &record
                                .salt
                                .clone()
                                .try_into()
                                .map_err(|_| DbError::SaltWrongSize)?,
                        )?;
                        let timestamp = record.timestamp.and_utc();
                        Ok((UserId(record.sender_id), Message::new(contents, timestamp)))
                    })
                    .collect::<Result<Vec<_>, DbError>>()?,
                res.first().unwrap().previous_message_id.map(MessageId),
            )),
            Err(e) => Err(e.into()),
        }
    }

    async fn get_querier<'a>(&'a self) -> Result<Self::Querier<'a>, Self::Error> {
        Ok(Querier {
            q: &self.pool,
            key: &self.suite,
            rng: &self.rng,
        })
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
        prod_id: &Self::ProductId,
    ) -> Result<Self::ConversationId, Self::Error> {
        let mut transaction = self.pool.begin().await?;

        let record = sqlx::query!(
            r#"
            SELECT id as "id!"
            FROM conversation
            WHERE ((client_id = ? AND seller_id = ?) OR (seller_id = ? AND client_id = ?)) AND product_id = ?;
        "#,
            my_id,
            their_id,
            my_id,
            their_id,
            prod_id,
        )
        .fetch_optional(&mut *transaction)
        .await?;

        if let Some(convo) = record {
            return Ok(ConversationId(convo.id));
        }

        let record = sqlx::query!(
            r#"
            INSERT INTO conversation (client_id, seller_id, product_id)
            VALUES (?, ?, ?)
            RETURNING id as "id!"
        "#,
            my_id,
            their_id,
            prod_id
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

        let (contents, salt) =
            CryptData::encrypt(msg.message().to_owned(), &self.suite, &mut self.rng)?;
        let salt = salt.to_vec();

        let timestamp = msg.timestamp().clone();

        let msg_id = sqlx::query!(
            r#"
            INSERT INTO message (content, salt, sender_id, conversation_id, previous_message_id, timestamp)
            VALUES (?, ?, ?, ?, ?,?)
            RETURNING id as "id!"
        "#,
            contents,
            salt,
            my_id,
            conversation,
            prev_id,
            timestamp,
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

        Ok(record.map(|i| i.last_message_id.map(MessageId))?)
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
                WHERE conversation.id = ? AND (client_id = ? OR seller_id = ?)
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
            "#,
            msg_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(ConversationId(record.conversation_id))
    }

    async fn get_product(&self, prod_id: &Self::ProductId) -> Result<Self::Product, Self::Error> {
        Ok(sqlx::query_as!(
            Product,
            r#"
                SELECT name as "name!", js_id as "jumpseller_id!", seller_id as "seller_id!" 
                FROM product
                WHERE product.id = ?
            "#,
            prod_id
        )
        .fetch_one(&self.pool)
        .await?)
    }

    async fn get_product_id_from_conversation_id(
        &self,
        conversation_id: &Self::ConversationId,
    ) -> Result<Self::ProductId, Self::Error> {
        let record = sqlx::query!(
            r#"
                SELECT product_id as "product_id!"
                FROM conversation
                WHERE id = ?
            "#,
            conversation_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(ProductId(record.product_id))
    }

    async fn add_product(
        &mut self,
        product: &Self::Product,
    ) -> Result<Self::ProductId, Self::Error> {
        let mut transaction = self.pool.begin().await?;

        let record = sqlx::query!(
            r#"
               SELECT id as "id!"
               FROM product
               WHERE js_id = ? 
            "#,
            product.jumpseller_id,
        )
        .fetch_optional(&mut *transaction)
        .await?;

        let prod = if let Some(r) = record {
            sqlx::query!(
                r#"
                    UPDATE product
                    SET name = ?, seller_id = ?
                    WHERE id = ?
                "#,
                product.name,
                product.seller_id,
                r.id
            )
            .execute(&mut *transaction)
            .await?;
            r.id.into()
        } else {
            let r = sqlx::query!(
                r#"
                    INSERT INTO product(name, js_id, seller_id)
                    VALUES(?,?,?)
                    RETURNING id as "id!"
                "#,
                product.name,
                product.jumpseller_id,
                product.seller_id
            )
            .fetch_one(&mut *transaction)
            .await?;
            r.id.into()
        };
        transaction.commit().await?;
        Ok(prod)
    }

    async fn belongs_to_seller(
        &self,
        seller_id: &Self::UserId,
        product_id: &Self::ProductId,
    ) -> Result<(), Self::Error> {
        let record = sqlx::query!(
            r#"
                SELECT seller_id
                FROM product
                WHERE id = ? AND seller_id = ?
            "#,
            product_id,
            seller_id
        )
        .fetch_optional(&self.pool)
        .await?;
        match record {
            Some(_) => Ok(()),
            None => Err(DbError::PermissionDenied),
        }
    }
}

#[cfg(test)]
mod test {
    use anyhow::anyhow;

    use crate::database::Database;
    use crate::database::crypto::CryptoKey;
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

        let password = "very_$ecure_and_$trong_P4$$w0rd_in_2025";
        let salt = "even_more_$ecure_$alt";
        let suite = CryptoKey::new(password, salt).map_err(|e| anyhow!("Error: {e}"))?;

        let mut db = SQLiteDB::new("sqlite::memory:", suite).await?;

        let alice_id = db.add_user(&alice).await?;
        let bob_id = db.add_user(&bob).await?;
        assert_ne!(alice_id, bob_id);

        let alice_again = db.add_user(&alice).await?;
        assert_eq!(alice_id, alice_again);

        let prod = Product {
            name: "Dill Dough".to_string(),
            seller_id: 1.into(),
            jumpseller_id: 1,
        };
        let prod_id = db.add_product(&prod).await?;

        let convo_id = db.start_conversation(&alice_id, &bob_id, &prod_id).await?;
        let same_id = db.start_conversation(&bob_id, &alice_id, &prod_id).await?;

        assert_eq!(convo_id, same_id);

        let hello = Message::from("Hello Bob!");
        let hello_id = db.post_msg(hello.clone(), &alice_id, &convo_id).await?;

        // Example queries
        let alice_bob_messages: Result<Vec<Message>, _> = {
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
            .fetch_one(querier.q)
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
            .fetch_one(querier.q)
            .await?;
            assert!(MessageId(msg_id.id) == hello_id);
            // Retrieve all messages between Alice and Bob
            let messages = sqlx::query!(
                r#"
                SELECT content as "content!", salt as "salt!"
                FROM message
                WHERE conversation_id = ?;
            "#,
                convo_id
            )
            .fetch_all(querier.q)
            .await?;
            messages
                .into_iter()
                .map(|m| -> Result<(CryptData<Message>, [u8; 12]), DbError> {
                    Ok((
                        CryptData::from(m.content),
                        m.salt.try_into().map_err(|_| DbError::SaltWrongSize)?,
                    ))
                })
                .map(|m| -> Result<Message, DbError> {
                    let (m, s) = m?;
                    Ok(m.decrypt(&querier.key, &s)?)
                })
                .collect()
        };

        assert_eq!(alice_bob_messages?, vec![hello]);

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
        let password = "very_$ecure_and_$trong_P4$$w0rd_in_2025";
        let salt = "even_more_$ecure_$alt";
        let suite = CryptoKey::new(password, salt).map_err(|e| anyhow!("Error: {e}"))?;

        let mut db = SQLiteDB::new("sqlite::memory:", suite).await?;

        let alice_id = db.add_user(&alice).await?;
        let bob_id = db.add_user(&bob).await?;
        assert_ne!(alice_id, bob_id);

        let alice_again = db.add_user(&alice).await?;
        assert_eq!(alice_id, alice_again);

        let prod = Product {
            name: "Dill Dough".to_string(),
            seller_id: 1.into(),
            jumpseller_id: 1,
        };
        let prod_id = db.add_product(&prod).await?;

        let convo_id = db.start_conversation(&alice_id, &bob_id, &prod_id).await?;

        // Test
        let last_msg = db.get_latest_message(&convo_id).await?;
        assert_eq!(last_msg, None);

        let hello = Message::from("Hello Bob!");
        let first_hello_id = db.post_msg(hello, &alice_id, &convo_id).await?;

        let last_msg = db.get_latest_message(&convo_id).await?;
        assert_eq!(last_msg, Some(first_hello_id));

        let hello = Message::from("Hello Alice!");
        let second_hello_id = db.post_msg(hello, &bob_id, &convo_id).await?;

        let last_msg = db.get_latest_message(&convo_id).await?.unwrap();
        assert_eq!(last_msg, second_hello_id);

        let (_, _, last_msg) = db.get_message(&last_msg).await?;
        assert_eq!(last_msg, Some(first_hello_id));
        Ok(())
    }
}
