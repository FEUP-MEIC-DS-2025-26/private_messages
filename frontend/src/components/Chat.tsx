"use client";

import useSWR from "swr";
import { Ref, useLayoutEffect, useRef, useState, useEffect } from "react";

// components
import ChatHeader from "./ChatHeader";
import UserMessage, { UserMessageProps } from "./UserMessage";
import MessageInput from "./MessageInput";

async function fetcher(URL: string) : Promise<any> {
  return await fetch(URL, { credentials: "include" }).then(res => res.json());
}

async function getMessages(URL: string, username: string) {
  const messages: any[] = await fetcher(`${URL}/recent`).then(({ content }) => content);
  return messages.map(message => ({
    isFromUser: message.sender_username === username,
    content: message.msg.contents
  }));
}

async function pollMessages(backendURL: string, stateMessageId: number, latestMessageId: number) {
  const newMessages = [];
  let currentId: number = latestMessageId;
  while (stateMessageId != currentId) {
    // The type is too cumbersome
    const message: any = await fetcher(`${backendURL}/api/chat/message/${currentId}`); 
    currentId = message.previous_msg;

    // This will also fetch messages this user sent, which is necessary to make sure they are displayed chronologically
    newMessages.push({
      isFromUser: false,
      content: message.content.msg.contents
    });
  }
  return newMessages;
}

// I don't know how to place this next to goToInbox
type fn = { (): void };
export default function Chat({ backendURL: string, id: number, username: string, goToInbox: fn }) {
  const url = `${backendURL}/api/chat/conversation/${id}`;

  const [ messageId, setMessageId ] : [ number, (i: number) => void ] = useState(-1);
  const [ messages, setMessages ] /*: [ UserMessageProps[], (ma: UserMessageProps[]) => void ]*/ = useState([]);

  useSWR(url, async (URL: string) => { 
    const messageId: number = await fetcher(`${URL}/latest`);
    setMessageId(messageId);

    const msgs: UserMessageProps[] = await getMessages(URL, username);
    setMessages(msgs);
  });

  const messageListRef: Ref<HTMLUListElement> = useRef(null);

  // automatically scroll the last message into view
  useLayoutEffect(() => {
    const messageList: null | HTMLUListElement = messageListRef.current;

    if (messageList) {
      setTimeout(() => {
        messageList.scrollTo({
            top: messageList.scrollHeight,
            behavior: "smooth",
          })
        }, 100);
    }
  }, [messages]);

  useEffect(() => {
    const intervalId = setInterval(async () => {
    
      // msgId is only -1 when the data hasn't been fetched yet. No matter the result, msgId cannot be -1 afterwards
      if (messageId != -1) {

        const latestMessageId: number = await fetcher(`${backendURL}/api/chat/conversation/${id}/latest`);
        if (latestMessageId !== null && latestMessageId !== undefined) {
          const newMessages = await pollMessages(backendURL, messageId, latestMessageId);

          /*
          const newMessages = [];
          let currentId = latestMessageId;
          while (messageId != currentId) {

            const message = await fetcher(`${backendURL}/api/chat/message/${currentId}`); 
            currentId = message.previous_msg;

            // This will also fetch messages this user sent, which is necessary to make sure they are displayed chronologically
            newMessages.push({
              isFromUser: false,
              content: message.content.msg.contents
            });
          }
          */

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

  const updateMessages = async (latestMessageId: number) => {
    const newMessages  = await pollMessages(backendURL, messageId, latestMessageId);
    /*
    const newMessages = [];
    let currentId = latestMessageId;
    while (messageId != currentId) {
      const message = await fetcher(`${backendURL}/api/chat/message/${currentId}`); 
      currentId = message.previous_msg;
      newMessages.push({
        isFromUser: false,
        content: message.content.msg.contents
      });
    }
    */
    newMessages.reverse();
    setMessageId(latestMessageId);
    setMessages([...messages, ...newMessages]);
  };

  return (
    <>
      <ChatHeader backendURL={backendURL} id={id} goToInbox={goToInbox} />
      {messages ? (
        <ul className="grow overflow-scroll flex flex-col gap-3 px-3" ref={messageListRef}>
          {messages.map((message: UserMessageProps, index: number) => (
            <li key={`message-${index}`}>
              <UserMessage {...message} />
            </li>
          ))}
        </ul>
      ) : (
        <div>Loading...</div>
      )}
      <MessageInput backendURL={backendURL} id={id} updateMessages={updateMessages}/>
    </>
  );
}
