pub mod crypto;
pub mod sqlite;

pub trait Database {
    type Error;
    type UserId;
    type UserProfile;
    type ConversationId;
    type MessageId;
    type Message;
    type Querier<'a>
    where
        Self: 'a;

    async fn get_conversations(
        &self,
        my_id: &Self::UserId,
    ) -> Result<Vec<Self::ConversationId>, Self::Error>;

    async fn get_peer(
        &self,
        my_id: &Self::UserId,
        conversation: &Self::ConversationId,
    ) -> Result<Self::UserId, Self::Error>;

    async fn get_user_id_from_username(&self, username: &str) -> Result<Self::UserId, Self::Error>;

    async fn get_user_profile(
        &self,
        their_id: &Self::UserId,
    ) -> Result<Self::UserProfile, Self::Error>;

    async fn get_message(
        &self,
        message: &Self::MessageId,
    ) -> Result<(Self::UserId, Self::Message, Option<Self::MessageId>), Self::Error>;

    async fn get_most_recent_messages(
        &self,
        conversation_id: &Self::ConversationId,
    ) -> Result<(Vec<(Self::UserId, Self::Message)>, Option<Self::MessageId>), Self::Error>;

    #[allow(dead_code)]
    async fn get_querier<'a>(&'a self) -> Result<Self::Querier<'a>, Self::Error>;

    async fn add_user(&mut self, profile: &Self::UserProfile) -> Result<Self::UserId, Self::Error>;

    async fn start_conversation(
        &mut self,
        my_id: &Self::UserId,
        their_id: &Self::UserId,
    ) -> Result<Self::ConversationId, Self::Error>;

    async fn post_msg(
        &mut self,
        msg: Self::Message,
        my_id: &Self::UserId,
        conversation: &Self::ConversationId,
    ) -> Result<Self::MessageId, Self::Error>;

    async fn get_latest_message(
        &self,
        conversation: &Self::ConversationId,
    ) -> Result<Option<Self::MessageId>, Self::Error>;

    async fn belongs_to_conversation(
        &self,
        id: &Self::UserId,
        conversation: &Self::ConversationId,
    ) -> Result<(), Self::Error>;

    async fn get_conversation_from_message(
        &self,
        msg_id: &Self::MessageId,
    ) -> Result<Self::ConversationId, Self::Error>;
}

/// Example implementation: Mock Database
#[allow(dead_code)]
pub mod mock {
    use anyhow::anyhow;
    use std::collections::HashMap;
    use tokio::sync::{RwLock, RwLockReadGuard};

    #[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct ConversationId(u64);

    #[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct UserId(u64);

    #[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct MessageId(u64);

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Message(String);

    #[derive(Debug, Clone)]
    pub struct UserProfile {
        username: String,
        name: String,
        age: u8,
    }

    pub struct MockDbInternal {
        users: HashMap<UserId, UserProfile>,
        /// (sender, receiver, last_message)
        conversations: HashMap<ConversationId, (UserId, UserId, Option<MessageId>)>,
        /// (sender, conversation, string content, previous_message)
        messages: HashMap<MessageId, (UserId, ConversationId, Message, Option<MessageId>)>,
    }

    pub struct MockDb {
        db: RwLock<MockDbInternal>,
    }

    impl super::Database for MockDb {
        type Error = anyhow::Error;

        type UserId = UserId;

        type UserProfile = UserProfile;

        type ConversationId = ConversationId;

        type MessageId = MessageId;

        type Message = Message;

        async fn get_conversations(
            &self,
            my_id: &Self::UserId,
        ) -> Result<Vec<Self::ConversationId>, Self::Error> {
            Ok(self
                .db
                .read()
                .await
                .conversations
                .iter()
                .filter(|(_, (u1, u2, _))| my_id == u1 || my_id == u2)
                .map(|(id, _)| *id)
                .collect())
        }

        async fn get_peer(
            &self,
            my_id: &Self::UserId,
            conversation: &Self::ConversationId,
        ) -> Result<Self::UserId, Self::Error> {
            match self.db.read().await.conversations.get(conversation) {
                Some(c) => {
                    if &c.0 == my_id {
                        Ok(c.1)
                    } else {
                        Ok(c.0)
                    }
                }
                None => Err(anyhow::anyhow!("Conversation not found")),
            }
        }

        async fn get_user_profile(
            &self,
            their_id: &Self::UserId,
        ) -> Result<Self::UserProfile, Self::Error> {
            match self.db.read().await.users.get(their_id) {
                Some(u) => Ok(u.clone()),
                None => Err(anyhow!("No such user")),
            }
        }

        async fn start_conversation(
            &mut self,
            my_id: &Self::UserId,
            their_id: &Self::UserId,
        ) -> Result<Self::ConversationId, Self::Error> {
            let my_convos = self.get_conversations(my_id).await?;
            for c in self.get_conversations(their_id).await? {
                if my_convos.contains(&c) {
                    return Ok(c);
                }
            }

            let mut querier = self.db.write().await;

            let id = querier
                .conversations
                .keys()
                .max_by_key(|x| x.0)
                .map(|x| x.0 + 1)
                .unwrap_or(0);
            let id = ConversationId(id);
            querier
                .conversations
                .insert(id.clone(), (*my_id, *their_id, None));
            Ok(id)
        }

        async fn post_msg(
            &mut self,
            msg: Self::Message,
            my_id: &Self::UserId,
            conversation: &Self::ConversationId,
        ) -> Result<Self::MessageId, Self::Error> {
            let mut querier = self.db.write().await;

            // Get the ID of the previous message
            let prev_id = *querier
                .conversations
                .get(conversation)
                .map(|(_, _, prev)| prev)
                .ok_or(anyhow!(
                    "Conversation with ID {conversation:?} was not found."
                ))?;

            // Get the highest ID and generate the next one
            let id = querier
                .messages
                .keys()
                .max_by_key(|x| x.0)
                .map(|x| x.0 + 1)
                .unwrap_or(0);
            let id = MessageId(id);

            // Write message
            querier
                .messages
                .insert(id.clone(), (*my_id, *conversation, msg, prev_id));

            // Change the last message pointer in the conversation table
            querier
                .conversations
                .get_mut(conversation)
                .map(|(_, _, prev)| *prev = Some(id));

            Ok(id)
        }

        async fn add_user(
            &mut self,
            profile: &Self::UserProfile,
        ) -> Result<Self::UserId, Self::Error> {
            if let Some(id) = self
                .db
                .read()
                .await
                .users
                .iter()
                .filter(|(_, prof)| prof.username == profile.username)
                .map(|(id, _)| id)
                .next()
            {
                return Ok(*id);
            }

            let mut querier = self.db.write().await;

            let id = querier
                .users
                .keys()
                .max_by_key(|x| x.0)
                .map(|x| x.0 + 1)
                .unwrap_or(0);
            let id = UserId(id);
            querier.users.insert(id.clone(), profile.clone());
            Ok(id)
        }

        type Querier<'a>
            = RwLockReadGuard<'a, MockDbInternal>
        where
            Self: 'a;

        async fn get_querier<'a>(&'a self) -> Result<Self::Querier<'a>, Self::Error> {
            Ok(self.db.read().await)
        }

        async fn get_message(
            &self,
            message: &Self::MessageId,
        ) -> Result<(Self::UserId, Self::Message, Option<Self::MessageId>), Self::Error> {
            match self.db.read().await.messages.get(message) {
                Some((usr, _conv, msg, prev)) => Ok((usr.clone(), msg.clone(), prev.clone())),
                None => Err(anyhow!("Message with ID {message:?} was not found.")),
            }
        }

        // FIXME: implement this for the mock
        async fn get_most_recent_messages(
            &self,
            conversation_id: &Self::ConversationId,
        ) -> Result<(Vec<(Self::UserId, Self::Message)>, Option<Self::MessageId>), Self::Error>
        {
            unimplemented!()
        }

        async fn get_latest_message(
            &self,
            conversation: &Self::ConversationId,
        ) -> Result<Option<MessageId>, Self::Error> {
            match self.db.read().await.conversations.get(conversation) {
                Some((_, _, prev)) => Ok(*prev),
                None => Err(anyhow!(
                    "Conversation with ID {conversation:?} was not found."
                )),
            }
        }

        async fn get_user_id_from_username(
            &self,
            username: &str,
        ) -> Result<Self::UserId, Self::Error> {
            self.db
                .read()
                .await
                .users
                .iter()
                .filter(|(_, p)| p.username == username)
                .map(|(id, _)| id)
                .next()
                .ok_or(anyhow!("No such user with username '{username}'."))
                .copied()
        }

        async fn belongs_to_conversation(
            &self,
            id: &Self::UserId,
            conversation: &Self::ConversationId,
        ) -> Result<(), Self::Error> {
            match self.db.read().await.conversations.get(conversation) {
                Some((id1, id2, _)) if id1 == id || id2 == id => Ok(()),
                _ => Err(anyhow!("No such conversation {conversation:?}.")),
            }
        }

        async fn get_conversation_from_message(
            &self,
            msg_id: &Self::MessageId,
        ) -> Result<Self::ConversationId, Self::Error> {
            match self.db.read().await.messages.get(msg_id) {
                Some((_, convo_id, _, _)) => Ok(*convo_id),
                None => Err(anyhow!("No such message {msg_id:?}.")),
            }
        }
    }

    impl Default for MockDbInternal {
        fn default() -> Self {
            Self {
                users: Default::default(),
                conversations: Default::default(),
                messages: Default::default(),
            }
        }
    }

    impl Default for MockDb {
        fn default() -> Self {
            Self {
                db: Default::default(),
            }
        }
    }

    #[cfg(test)]
    mod test {
        use crate::database::Database;
        use crate::database::mock::*;

        #[tokio::test]
        async fn test_mock() -> anyhow::Result<()> {
            let alice = UserProfile {
                username: "alice_11".to_owned(),
                name: "Alice Arnold".to_owned(),
                age: 24,
            };

            let bob = UserProfile {
                username: "bobert22".to_owned(),
                name: "Bob Bellows".to_owned(),
                age: 47,
            };

            let mut db = MockDb::default();

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

            // Example queries. Notice it is inside a scope, so that the querier lock automatically frees.
            let alice_bob_messages = {
                let querier = db.get_querier().await?;
                // Count all messages between Alice and Bob
                let msg_count = querier
                    .messages
                    .values()
                    .filter(|(_, c, _, _)| c == &convo_id)
                    .count();
                assert_eq!(msg_count, 1);
                // Make sure that the only message is the 'Hello Bob!' one.
                let msg_id = querier
                    .messages
                    .iter()
                    .filter(|(_, (_, c, _, _))| c == &convo_id)
                    .map(|(id, _)| id)
                    .all(|x| x == &hello_id);
                assert!(msg_id);
                // Retrieve all messages between Alice and Bob
                let messages = querier
                    .messages
                    .values()
                    .filter(|(_, c, _, _)| c == &convo_id)
                    .map(|(_, _, m, _)| m.clone())
                    .collect::<Vec<_>>();
                messages
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
                age: 24,
            };

            let bob = UserProfile {
                username: "bobert22".to_owned(),
                name: "Bob Bellows".to_owned(),
                age: 47,
            };

            let mut db = MockDb::default();

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
}
