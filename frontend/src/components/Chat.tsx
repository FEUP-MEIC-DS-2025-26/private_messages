"use client";

import useSWR from "swr";
import { Ref, useLayoutEffect, useRef, useState, useEffect } from "react";

// components
import ChatHeader from "./ChatHeader";
import UserMessage, { UserMessageProps } from "./UserMessage";
import MessageInput from "./MessageInput";
import { Divider } from "@mui/material";

const fetcher = (URL: string) =>
  fetch(URL, { credentials: "include" }).then((res) => res.json());

/**
 * Fetches the chat messages from the backend.
 * @param {string} URL - the URL
 * @param {string} username - the user's username
 */
const getMessages = async (URL: string, username: string) => {
  const messages: any[] = await fetcher(`${URL}/recent`).then(
    ({ content }) => content
  );

  return messages.map((message) => ({
    isFromUser: message.sender_username === username,
    content: message.msg.contents,
  }));
};

interface ChatProps {
  backendURL: string;
  id: number;
  username: string;
  goToInbox: () => void;
}

/**
 * A private conversation between two users.
 */
export default function Chat({
  backendURL,
  id,
  username,
  goToInbox,
}: ChatProps) {
  /*
  const { data: msgs } = useSWR(
    `${backendURL}/api/chat/conversation/${id}`,
    (URL) => getMessages(URL, username)
  );

  const { data: latestMsgId } = useSWR(
    `${backendURL}/api/chat/conversation/${id}`,
    URL => fetcher(`${URL}/latest`).then(({ id }) => id)
  );
  */

  const url = `${backendURL}/api/chat/conversation/${id}`;

  const [messageId, setMessageId] = useState(-1);
  const [messages, setMessages] = useState([]);

  // For some reason, these two need to be in the same function
  useSWR(url, async (URL) => {
    const messageId = await fetcher(`${URL}/latest`);
    setMessageId(messageId);

    const msgs = await getMessages(URL, username);
    setMessages(msgs);
  });

  const messageListRef: Ref<HTMLUListElement> = useRef(null);

  // automatically scroll the last message into view
  useLayoutEffect(() => {
    const messageList = messageListRef.current;

    if (messageList) {
      setTimeout(() => {
        messageList.scrollTo({
          top: messageList.scrollHeight,
          behavior: "smooth",
        });
      }, 100);
    }
  }, [messages]);

  useEffect(() => {
    const intervalId = setInterval(async () => {
      // msgId is only -1 when the data hasn't been fetched yet. No matter the result, msgId cannot be -1 afterwards
      if (messageId != -1) {
        const latestMessageId = await fetcher(
          `${backendURL}/api/chat/conversation/${id}/latest`
        );
        if (latestMessageId !== null && latestMessageId !== undefined) {
          const newMessages = [];
          let currentId = latestMessageId;
          while (messageId != currentId) {
            const message = await fetcher(
              `${backendURL}/api/chat/message/${currentId}`
            );
            currentId = message.previous_msg;

            // This will also fetch messages this user sent, which is necessary to make sure they are displayed chronologically
            newMessages.push({
              isFromUser: false,
              content: message.content.msg.contents,
            });
          }

          if (newMessages.length > 0) {
            /*
             * The first message is the latest one, the second one is the second-to-latest and so on
             * Reversing the array sorts the messages chronologically - no need for timestamps
             */
            newMessages.reverse();
            setMessageId(latestMessageId);
            setMessages([...messages, ...newMessages]);
          }
        }
      }
    }, 5000);
    return () => clearInterval(intervalId);
  }, [messages]);

  return (
    <>
      {/** Header */}
      <ChatHeader backendURL={backendURL} id={id} goToInbox={goToInbox} />
      <Divider />

      {/** Chat */}
      {messages ? (
        <ul
          className="grow overflow-scroll flex flex-col gap-3 px-3"
          ref={messageListRef}
        >
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
      <MessageInput backendURL={backendURL} id={id} />
    </>
  );
}
