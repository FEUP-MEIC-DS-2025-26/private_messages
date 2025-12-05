import { Divider, List, ListItem } from '@mui/material';
import { Fragment } from 'react/jsx-runtime';
import { useState } from 'react';

// components
import ChatPreview from './ChatPreview';
import SearchBar from './SearchBar';

interface InboxProps {
  backendURL: string;
  userID: number;
  goToChat: (id: number) => void;
}

interface ChatStateProps {
  id: number;
  userID: number;
  name: string;
  username: string;
  lastMessage: string;
  profilePictureURL: string;
  unreadMessages: number;
  product: string;
  visible: boolean;
}

const fetcher = (URL: string) => fetch(URL, { credentials: "include" }).then(res => res.json());

async function getChats(URL: string, userID: number) : Promise<ChatStateProps[]> {
  // login
  const response = await fetch(`${URL}/login?id=${userID}`, { credentials: "include" });
  if (!response.ok) {
    console.error("Error when logging in");
    return [];
  }

  // fetch conversations
  const conversationIDs: number[] = await fetcher(`${URL}/conversation`);

  // fetch peers' usernames
  const userIDs: number[] = await Promise.all(
    conversationIDs.map(
      (id: number) => fetcher(`${URL}/conversation/${id}/peer`).then(({ id }) => id)
    )
  );

  // fetch peers' usernames and display names
  const names: { username: string; name: string }[] = await Promise.all(
    userIDs.map((id: number) =>
      fetcher(`${URL}/user/${id}`).then(({ username, name }) => ({ username, name }))
    ),
  );

  // fetch the last message from each conversation
  const lastMessages: string[] = await Promise.all(
    conversationIDs.map((id: number) =>
      fetcher(`${URL}/conversation/${id}/latest`)
        .then(({ id }) => id)
        .then((messageID: number) => fetcher(`${URL}/message/${messageID}`))
        .then(message => message.content.msg.contents),
    )
  );

  const products: string[] = await Promise.all(
    conversationIDs.map((id: number) =>
      fetcher(`${URL}/conversation/${id}/product`)
        .then(({ id }) => id)
        .then((productId: number) => fetcher(`${URL}/product/${productId}`))
        .then(({ name }) => name)
    )
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
    visible: true
  }));
};

export default function Inbox({ backendURL, userID, goToChat }: InboxProps) {
  const [ chats, setChats ] = useState<ChatStateProps[]>([]); 

  if (chats.length == 0) {
    getChats(`${backendURL}/api/chat`, userID).then(chacha20_poly1305 => { setChats(chacha20_poly1305); });
  }

  const filterChats = (text: string) => {
    setChats(chats.map(chat => ({...chat, visible: chat.name.includes(text) || chat.username.includes(text) })));
  };

  return (
    <>
      <SearchBar filter={filterChats}/>
      <List sx={{ width: 1 }}>
        {chats.filter(({ visible }) => visible).map((chat: ChatStateProps, index: number) => (
          <Fragment key={`chat-${chat.id}`}>
            <ListItem onClick={() => goToChat(chat.id)} sx={{ py: 0 }}>
              <ChatPreview {...chat} />
            </ListItem>
            {index + 1 < chats.length && (
              <Divider variant="middle" component="li" aria-hidden />
            )}
          </Fragment>
        ))}
      </List>
    </>
  );
}
