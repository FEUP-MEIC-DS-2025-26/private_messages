pub trait Database {
    type Error;
    type UserId;
    type UserProfile;
    type ConversationId;
    type MessageId;
    type Message;

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
    use tokio::sync::RwLock;

    use crate::database::Database;

    #[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct ConversationId(u64);
    #[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct UserId(u64);
    #[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct MessageId(u64);

    #[derive(Debug, Clone)]
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

            let id = self
                .db
                .read()
                .await
                .conversations
                .keys()
                .max_by_key(|x| x.0)
                .map(|x| x.0 + 1)
                .unwrap_or(0);
            let id = ConversationId(id);
            self.db
                .write()
                .await
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
            let id = self
                .db
                .read()
                .await
                .messages
                .keys()
                .max_by_key(|x| x.0)
                .map(|x| x.0 + 1)
                .unwrap_or(0);
            let id = MessageId(id);
            self.db
                .write()
                .await
                .messages
                .insert(id.clone(), (*my_id, *conversation, msg));
            Ok(id)
        }

        async fn add_user(
            &mut self,
            profile: &Self::UserProfile,
        ) -> Result<Self::UserId, Self::Error> {
            let id = self
                .db
                .read()
                .await
                .users
                .keys()
                .max_by_key(|x| x.0)
                .map(|x| x.0 + 1)
                .unwrap_or(0);
            let id = UserId(id);
            self.db
                .write()
                .await
                .users
                .insert(id.clone(), profile.clone());
            Ok(id)
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

        let convo_id = db.start_conversation(&alice_id, &bob_id).await?;
        let same_id = db.start_conversation(&bob_id, &alice_id).await?;

        assert_eq!(convo_id, same_id);

        Ok(())
    }
}
