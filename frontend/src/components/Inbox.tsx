import { Divider, List, ListItem } from '@mui/material';
import { Fragment } from 'react/jsx-runtime';
import useSWR from 'swr';

// components
import ChatPreview, { ChatPreviewProps } from './ChatPreview';
import PeerSearch from './PeerSearch';

const fetcher = (URL: string) => fetch(URL, { credentials: "include" }).then(res => res.json());

const getChats = async (URL: string, userID: number) => {
  // login
  const response = await fetch(`${URL}/login?id=${userID}`, { credentials: "include" });
  if (!response.ok) {
    console.error("Error when logging in");
    return;
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
  const { data: chats, isLoading, mutate } = useSWR(
    `${backendURL}/api/chat/conversation`,
    () => getChats(`${backendURL}/api/chat`, userID),
  );

  if (isLoading || !chats) {
    return <div>Loading...</div>;
  }

  const filterChats = text => {
    mutate(chats.map(chat => ({...chat, visible: chat.name.includes(text) || chat.username.includes(text) })));
  };

  return (
    <>
      <PeerSearch filter={filterChats}/>
      <List sx={{ width: 1 }}>
        {chats.filter(({ visible }) => visible).map((chat: ChatPreviewProps, index: number) => (
          <Fragment key={`chat-${chat.id}`}>
            {/** chat preview */}
            <ListItem onClick={() => goToChat(chat.id)} sx={{ py: 0 }}>
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
