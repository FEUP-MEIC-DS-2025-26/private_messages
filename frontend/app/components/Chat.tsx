'use client';

import useSWR from 'swr';

// components
import ChatHeader from './ChatHeader';
import UserMessage, { UserMessageProps } from './UserMessage';
import MessageInput from './MessageInput';

/**
 * A function for fetching data from the backend.
 * @param {string} URL - the URL
 */
const fetcher = (URL: string) => fetch(URL).then((res) => res.json());

/**
 * Fetches the chat messages from the backend.
 * @param {string} URL - the URL
 */
const getMessages = async (URL: string, username: string) => {
  const messages: any[] = await fetcher(URL).then((message) => message.content);
  return messages.map((message) => ({
    isFromUser: message.sender_username === username,
    content: message.msg,
  }));
};

interface ChatProps {
  /** The unique chat identifier. */
  id: number;
  /** The user's username. */
  username: string;
  /** A function for navigating to the inbox. */
  goToInbox: () => void;
}

/**
 * A private conversation between two users.
 */
export default function Chat({ id, username, goToInbox }: ChatProps) {
  const { data: messages } = useSWR(
    `/api/chat/conversation/${id}/recent`,
    (URL) => getMessages(URL, username),
  );

  return (
    <>
      {/** Header */}
      <ChatHeader id={id} goToInbox={goToInbox} />

      {/** Chat */}
      {messages ? (
        <ul className="grow overflow-scroll flex flex-col gap-3 px-3">
          {messages.map((message: UserMessageProps, index: number) => (
            <li key={`message-${index}`}>
              <UserMessage {...message} />
            </li>
          ))}
        </ul>
      ) : (
        <div>Loading...</div>
      )}

      {/* Text bar */}
      <MessageInput />
    </>
  );
}
