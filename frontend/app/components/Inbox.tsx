'use client';

import useSWR from 'swr';
import ChatPreview, { ChatPreviewProps } from './UserMessagePreview';

interface InboxProps {
  username: string;
  setChat: (id: number) => void;
}

/**
 * A function for fetching the user's conversations from the server.
 * @param {string} username - the user's username
 * @returns the user's conversations
 */
const getChats = async (username: string) => {
  const API_URL = '/api/chat';

  // login
  await fetch(`${API_URL}/login?username=${username}`);

  // fetch the conversations
  const conversationIDs: number[] = await fetch(`${API_URL}/conversation`).then(
    (res) => res.json(),
  );

  // fetch the usernames of the peers with whom we are conversing
  const usernames: string[] = await Promise.all(
    conversationIDs.map((id: number) =>
      fetch(`${API_URL}/conversation/${id}/peer`).then((res) => res.json()),
    ),
  );

  // fetch the peers' display names
  const fullNames: string[] = await Promise.all(
    usernames.map((username: string) =>
      fetch(`${API_URL}/user/${username}`)
        .then((res) => res.json())
        .then((user) => user.name),
    ),
  );

  // fetch the last message from each conversation
  const lastMessages: string[] = await Promise.all(
    conversationIDs.map((id: number) =>
      fetch(`${API_URL}/conversation/${id}/latest`)
        .then((res) => res.json())
        .then((id: number) => fetch(`${API_URL}/message/${id}`))
        .then((res) => res.json())
        .then((message) => message.content.msg),
    ),
  );

  // create an array with the conversations
  return conversationIDs.map((id: number, index: number) => ({
    id,
    username: usernames[index],
    name: fullNames[index],
    lastMessage: lastMessages[index],
    profilePictureURL: 'https://thispersondoesnotexist.com/',
    unreadMessages: Math.floor(Math.random() * 10),
  }));
};

/**
 * The user's inbox.
 */
export default function Inbox({ username, setChat }: InboxProps) {
  const { data: chats, isLoading } = useSWR('/api/chat/conversation', () =>
    getChats(username),
  );

  if (isLoading || !chats) {
    return <div>Loading...</div>;
  }

  return (
    <ul className="flex flex-col *:not-last:border-b">
      {chats.map((chat: ChatPreviewProps) => (
        <li key={`chat-${chat.id}`}>
          <button onClick={() => setChat(chat.id)}>
            <ChatPreview {...chat} />
          </button>
        </li>
      ))}
    </ul>
  );
}
