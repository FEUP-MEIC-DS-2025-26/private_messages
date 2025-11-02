'use client';

import useSWR from 'swr';

// components
import ChatHeader from './ChatHeader';
import UserMessage from './UserMessage';
import MessageInput from './MessageInput';

/**
 * A function for fetching data from the backend.
 * @param {string} URL - the URL
 */
const fetcher = (URL: string) => fetch(URL).then((res) => res.json());

/**
 * A private conversation between two users.
 */
export default function Chat({ id }: { id: number }) {
  const { data: messages } = useSWR(
    `/api/chat/conversation/${id}/recent`,
    fetcher,
  );

  return (
    <>
      {/** Header */}
      <ChatHeader id={id} />

      {/** Chat */}
      {/* <ul className="grow overflow-scroll flex flex-col gap-3 px-3" ref={ul}>
        {messages.map(({ isFromUser, content }, index) => (
          <li key={`${content}-${index}`}>
            <UserMessage isFromUser={isFromUser} content={content} />
          </li>
        ))}
      </ul> */}

      {/* Text bar */}
      <MessageInput />
    </>
  );
}
