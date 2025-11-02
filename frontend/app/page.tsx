'use client';

import useSWR from 'swr';
import ConversationPreview, {
  ConversationPreviewProps,
} from './components/UserMessagePreview';

const USERNAME: string = "john";

/**
 * A function for fetching the user's conversations from the server.
 * @param {string} username - the user's username
 * @returns the user's conversations
 */
const getConversations = async (username: string) => {
  const API_URL = '/api/chat';

  // login
  await fetch(`${API_URL}/login?username=${username}`);

  // fetch the conversations
  const conversationIDs: number[] = await fetch(`${API_URL}/conversation`).then(
    (res) => res.json(),
  );

  // fetch the usernames of the users with whom we are conversing
  const usernames: string[] = await Promise.all(
    conversationIDs.map((id: number) =>
      fetch(`${API_URL}/conversation/${id}/peer`).then((res) => res.json()),
    ),
  );

  // fetch the users' full names
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
export default function Inbox() {
  const { data: conversations, isLoading } = useSWR(
    '/api/chat/conversation',
    () => getConversations(USERNAME),
  );

  if (isLoading || !conversations) {
    return <div>Loading...</div>;
  }

  return (
    <ul className="flex flex-col *:not-last:border-b">
      {conversations.map((conversation: ConversationPreviewProps) => (
        <li key={`conversation-${conversation.id}`}>
          <ConversationPreview {...conversation} />
        </li>
      ))}
    </ul>
  );
}
