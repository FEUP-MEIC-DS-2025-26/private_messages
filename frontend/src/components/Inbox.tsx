import { Button, Divider, List, ListItem } from '@mui/material';
import useSWR from 'swr';

// components
import ChatPreview, { ChatPreviewProps } from './ChatPreview';

/**
 * A function for fetching data from the backend.
 * @param {string} URL - the URL
 */
const fetcher = (URL: string) =>
  fetch(URL, { credentials: 'include' }).then((res) => res.json());

/**
 * A function for fetching the user's conversations from the server.
 * @param {string} URL - the URL
 * @param {string} userID - the user's JumpSeller ID
 * @returns the user's conversations
 */
const getChats = async (URL: string, userID: number) => {
  // login
  await fetch(`${URL}/login?id=${userID}`, {
    credentials: 'include',
  });

  // fetch the conversations
  const conversationIDs: number[] = await fetch(`${URL}/conversation`, {
    credentials: 'include',
  }).then((res) => res.json());

  // fetch the peers' usernames
  const userIDs: number[] = await Promise.all(
    conversationIDs.map((id: number) =>
      fetcher(`${URL}/conversation/${id}/peer`),
    ),
  );

  // fetch the peers' display names
  const fullNames: string[] = await Promise.all(
    userIDs.map((id: number) =>
      fetcher(`${URL}/user/${id}`).then((user) => user.name),
    ),
  );

  // fetch the last message from each conversation
  const lastMessages: string[] = await Promise.all(
    conversationIDs.map((id: number) =>
      fetcher(`${URL}/conversation/${id}/latest`)
        .then((messageID: number) => fetcher(`${URL}/message/${messageID}`))
        .then((message) => message.content.msg.contents),
    ),
  );

  const products: string[] = await Promise.all(
    conversationIDs.map((id: number) =>
      fetcher(`${URL}/conversation/${id}/product`)
        .then((productId: number) => fetcher(`${URL}/product/${productId}`))
        .then((product) => product.name),
    ),
  );

  // create an array with the conversations
  return conversationIDs.map((id: number, index: number) => ({
    id,
    userID: userIDs[index],
    name: fullNames[index],
    lastMessage: lastMessages[index],
    profilePictureURL: 'https://thispersondoesnotexist.com/',
    unreadMessages: Math.floor(Math.random() * 10),
    product: products[index],
  }));
};

interface InboxProps {
  /** The URL that points to the backend. */
  backendURL: string;
  /** The user's JumpSeller ID. */
  userID: number;
  /**
   * A function for navigating to a chat.
   * @param {number} id - the unique chat identifier
   */
  goToChat: (id: number) => void;
}

/**
 * The user's inbox.
 */
export default function Inbox({ backendURL, userID, goToChat }: InboxProps) {
  const { data: chats, isLoading } = useSWR(
    `${backendURL}/api/chat/conversation`,
    () => getChats(`${backendURL}/api/chat`, userID),
  );
  // const chats = await getChats(`${backendURL}/api/chat`, username);

  if (isLoading || !chats) {
    return <div>Loading...</div>;
  }

  return (
    <List sx={{ width: 1 }}>
      {chats.map((chat: ChatPreviewProps, index: number) => (
        <>
          {/** chat preview */}
          <ListItem
            key={`chat-${chat.id}`}
            onClick={() => goToChat(chat.id)}
            sx={{ py: 0 }}
          >
            <ChatPreview {...chat} />
          </ListItem>

          {/** divider */}
          {index + 1 < chats.length && (
            <Divider variant="middle" component="li" aria-hidden />
          )}
        </>
      ))}
    </List>
  );
}
