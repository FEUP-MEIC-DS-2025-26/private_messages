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

    async fn get_user_profile(
        &self,
        their_id: &Self::UserId,
    ) -> Result<Self::UserProfile, Self::Error>;

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
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use anyhow::anyhow;
    use tokio::sync::{RwLock, RwLockReadGuard};

    use crate::database::Database;

    #[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct ConversationId(u64);
    #[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct UserId(u64);
    #[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct MessageId(u64);

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    struct Message(String);
    #[derive(Debug, Clone)]
    struct UserProfile {
        username: String,
        name: String,
        age: u8,
    }

    struct MockDbInternal {
        users: HashMap<UserId, UserProfile>,
        conversations: HashMap<ConversationId, (UserId, UserId)>,
        messages: HashMap<MessageId, (UserId, ConversationId, Message)>,
    }

    struct MockDb {
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
                .filter(|(_, (u1, u2))| my_id == u1 || my_id == u2)
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
                .insert(id.clone(), (*my_id, *their_id));
            Ok(id)
        }

        async fn post_msg(
            &mut self,
            msg: Self::Message,
            my_id: &Self::UserId,
            conversation: &Self::ConversationId,
        ) -> Result<Self::MessageId, Self::Error> {
            let mut querier = self.db.write().await;

            let id = querier
                .messages
                .keys()
                .max_by_key(|x| x.0)
                .map(|x| x.0 + 1)
                .unwrap_or(0);
            let id = MessageId(id);
            querier
                .messages
                .insert(id.clone(), (*my_id, *conversation, msg));
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
                .filter(|(_, c, _)| c == &convo_id)
                .count();
            assert_eq!(msg_count, 1);
            // Make sure that the only message is the 'Hello Bob!' one.
            let msg_id = querier
                .messages
                .iter()
                .filter(|(_, (_, c, _))| c == &convo_id)
                .map(|(id, _)| id)
                .all(|x| x == &hello_id);
            assert!(msg_id);
            // Retrieve all messages between Alice and Bob
            let messages = querier
                .messages
                .values()
                .filter(|(_, c, _)| c == &convo_id)
                .map(|(_, _, m)| m.clone())
                .collect::<Vec<_>>();
            messages
        };

        assert_eq!(alice_bob_messages, vec![Message("Hello Bob!".to_owned())]);

        Ok(())
    }
}
