'use client';

import useSWR from 'swr';
import UserMessagePreview from './components/UserMessagePreview';

// hard-coded user (only for the prototype)
const USERNAME = 'john';

/**
 * A function for fetching the user's conversations from the server.
 * @returns the user's conversations
 */
const getConversations = async (username: string) => {
  // login
  await fetch(`/api/chat/login?username=${username}`);

  // fetch the conversations
  const conversationIDs: number[] = await fetch('/api/chat/conversation').then(
    (res) => res.json(),
  );

  // fetch the usernames of the users with whom we are conversing
  const usernames: string[] = await Promise.all(
    conversationIDs.map((id: number) =>
      fetch(`/api/chat/conversation/${id}/peer`).then((res) => res.json()),
    ),
  );

  // fetch the user profiles
  const userProfiles: { username: string; name: string }[] = await Promise.all(
    usernames.map((username: string) =>
      fetch(`api/chat/user/${username}`).then((res) => res.json()),
    ),
  );

  // fetch the last message from each conversation
  const lastMessage: string[] = await Promise.all(
    conversationIDs.map((id: number) =>
      fetch(`/api/chat/conversation/${id}/latest`).then((res) => res.json()),
    ),
  );

  // create an array with the conversations
  const conversations: any[] = [];

  for (let index = 0; index < conversationIDs.length; ++index) {
    conversations.push({
      id: conversationIDs[index],
      username: usernames[index],
      name: userProfiles[index].name,
      lastMessage: lastMessage[index],
    });
  }

  return conversations;
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
      {conversations.map((conversation: any) => (
        <li key={`conversation-${conversation.id}`}>
          <UserMessagePreview
            {...conversation}
            profilePictureURL="https://thispersondoesnotexist.com/"
            unreadMessages={2}
            lastMessageDate="21/05/2026"
          />
        </li>
      ))}
    </ul>
  );
}
