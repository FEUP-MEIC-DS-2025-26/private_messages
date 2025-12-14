import { Divider, List, ListItem } from '@mui/material';
import { Fragment } from 'react/jsx-runtime';
import { useEffect, useState } from 'react';

import ChatPreview, { ChatPreviewProps } from './ChatPreview';
import SearchBar from './SearchBar';
import { UserMessageProps } from './UserMessage';
import ErrorPage from './ErrorPage';
import { fetcher, login } from '../utils';

interface InboxProps {
  /** The URL that points to the backend. */
  backendURL: string;
  /** The user's JumpSeller ID. */
  userID: number;
}

async function getChats(
  URL: string,
  userID: number,
): Promise<ChatPreviewProps[]> {
  // login
  await login(URL, userID);

  // fetch the conversations
  const conversationIDs: number[] = await fetcher(`${URL}/conversation`);

  // fetch peers' usernames
  const userIDs: number[] = await Promise.all(
    conversationIDs.map((id: number) =>
      fetcher(`${URL}/conversation/${id}/peer`).then(({ id }) => id),
    ),
  );

  // fetch peers' usernames and display names
  const names: { username: string; name: string }[] = await Promise.all(
    userIDs.map((id: number) =>
      fetcher(`${URL}/user/${id}`).then(({ username, name }) => ({
        username,
        name,
      })),
    ),
  );

  // fetch the last message from each conversation
  const lastMessages: UserMessageProps[] = await Promise.all(
    conversationIDs.map((id: number) =>
      fetcher(`${URL}/conversation/${id}/latest`)
        .then(({ id }) => id)
        .then((messageID: number) => fetcher(`${URL}/message/${messageID}`))
        .then(({ content }) => content)
        .then((content) => {
          const message = content.msg;

          return {
            isFromUser: content.sender_jsid === userID,
            content: message.contents,
            timestamp: new Date(message.timestamp),
            visible: true,
          };
        }),
    ),
  );

  const products: string[] = await Promise.all(
    conversationIDs.map((id: number) =>
      fetcher(`${URL}/conversation/${id}/product`)
        .then(({ id }) => id)
        .then((productId: number) => fetcher(`${URL}/product/${productId}`))
        .then(({ name }) => name),
    ),
  );

  // create an array with the conversations
  return conversationIDs.map((id: number, index: number) => ({
    id,
    userID: userIDs[index],
    ...names[index],
    lastMessage: lastMessages[index],
    profilePictureURL: 'https://thispersondoesnotexist.com/',
    unreadMessages: Math.floor(Math.random() * 10),
    product: products[index],
    visible: true,
  }));
}

export default function Inbox({ backendURL, userID }: InboxProps) {
  const [chats, setChats] = useState<ChatPreviewProps[] | null>(null);
  const [chatError, setChatError] = useState<Error | null>(null);

  useEffect(() => {
    if (chats == null) {
      try {
        getChats(`${backendURL}/api/chat`, userID).then((chacha20_poly1305) => {
          setChats(chacha20_poly1305);
        });
      } catch (e: any) {
        setChatError(e);
      }
    }
  }, []);
  
  if (chatError != null) {
    return <ErrorPage
      message="It seems you are not supposed to see this..."
      error={chatError}
      redirectURL={`${backendURL}/auth`}
    />
  }

  const filterChats = (text: string) => {
    text = text.toLowerCase();

    setChats(
      chats.map((chat) => ({
        ...chat,
        visible:
          chat.name.toLowerCase().includes(text) ||
          chat.username.toLowerCase().includes(text) ||
          chat.product.toLowerCase().includes(text),
      })),
    );
  };

  return (
    <>
      <SearchBar filter={filterChats} />
      <List sx={{ width: 1 }}>
        {chats
          .filter(({ visible }) => visible)
          .map((chat: ChatPreviewProps, index: number) => (
            <Fragment key={`chat-${chat.id}`}>
              {/** chat preview */}
              <ListItem sx={{ py: 0 }}>
                <ChatPreview {...chat} />
              </ListItem>

              {/** divider */}
              {index + 1 < chats.length && (
                <Divider variant="middle" component="li" aria-hidden />
              )}
            </Fragment>
          ))}
      </List>
    </>
  );
}
