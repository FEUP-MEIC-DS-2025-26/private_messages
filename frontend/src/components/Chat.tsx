import { Box } from '@mui/material';
import { Ref, useLayoutEffect, useRef, useState, useEffect } from 'react';
import useSWR from 'swr';

// components
import ChatHeader from './ChatHeader';
import UserMessage, { UserMessageProps } from './UserMessage';
import MessageInput from './MessageInput';
import { Divider, List, ListItem } from '@mui/material';

const fetcher = (URL: string) =>
  fetch(URL, { credentials: 'include' }).then((res) => res.json());

/**
 * Fetches the chat messages from the backend.
 * @param {string} URL - the URL
 * @param {string} userID - the user's JumpSeller ID
 */
const getMessages = async (URL: string, userID: number) => {
  const messages: any[] = await fetcher(`${URL}/recent`).then(
    ({ content }) => content,
  );

  return messages.map((message) => ({
    isFromUser: message.sender_jsid === userID,
    content: message.msg.contents,
    timestamp: new Date(message.msg.timestamp),
  }));
};

async function pollMessages(
  backendURL: string,
  userID: number,
  stateMessageId: number,
  latestMessageId: number,
): Promise<UserMessageProps[] | null> {
  const newMessages = [];
  let currentId: number = latestMessageId;
  while (stateMessageId != currentId) {
    const message: any = await fetcher(
      `${backendURL}/api/chat/message/${currentId}`,
    );

    currentId = message.previous_msg;
    const messageContent = message.content;

    // This will also fetch messages this user sent, which is necessary to make sure they are displayed chronologically
    newMessages.push({
      isFromUser: messageContent.sender_jsid === userID,
      content: messageContent.msg.contents,
      timestamp: new Date(messageContent.msg.timestamp),
    });
  }

  return newMessages;
}

interface ChatProps {
  backendURL: string;
  id: number;
  userID: number;
  goToInbox: () => void;
}

/**
 * A private conversation between two users.
 */
export default function Chat({ backendURL, id, userID, goToInbox }: ChatProps) {
  const url = `${backendURL}/api/chat/conversation/${id}`;

  const messageListRef: Ref<HTMLUListElement> = useRef(null);
  const [messageId, setMessageId] = useState<number>(-1);
  const [messages, setMessages] = useState<UserMessageProps[]>([]);

  // For some reason, these two need to be in the same function
  useSWR(url, async (URL: string) => {
    const messageId: number = await fetcher(`${URL}/latest`).then((x) => x.id);
    setMessageId(messageId);

    const msgs: UserMessageProps[] = await getMessages(URL, userID);
    setMessages(msgs);
  });

  // automatically scroll the last message into view
  useLayoutEffect(() => {
    const messageList = messageListRef.current;

    if (messageList) {
      setTimeout(() => {
        messageList.scrollTo({
          top: messageList.scrollHeight,
          behavior: 'smooth',
        });
      }, 100);
    }
  }, [messages]);

  useEffect(() => {
    const intervalId = setInterval(async () => {
      // msgId is only -1 when the data hasn't been fetched yet. No matter the result, msgId cannot be -1 afterwards
      if (messageId != -1) {
        const latestMessageId: number = await fetcher(
          `${backendURL}/api/chat/conversation/${id}/latest`,
        ).then((x) => x.id);
        if (latestMessageId !== null && latestMessageId !== undefined) {
          const newMessages = await pollMessages(
            backendURL,
            userID,
            messageId,
            latestMessageId,
          );

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
    const newMessages: UserMessageProps[] = await pollMessages(
      backendURL,
      userID,
      messageId,
      latestMessageId,
    );
    newMessages.reverse();
    setMessageId(latestMessageId);
    setMessages([...messages, ...newMessages]);
  };

  return (
    <Box
      sx={{
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
      }}
    >
      {/* Header */}
      <ChatHeader backendURL={backendURL} id={id} goToInbox={goToInbox} />
      <Divider />

      {/* Chat */}
      {messages ? (
        <List
          ref={messageListRef}
          sx={{
            display: 'flex',
            flexDirection: 'column',
            py: '16px',
            maxHeight: '100%',
            gap: '8px',
            flexGrow: 1,
            overflow: 'auto',
          }}
        >
          {messages.map((message: UserMessageProps, index: number) => (
            <ListItem key={`message-${index}`} sx={{ py: 0 }}>
              <UserMessage {...message} />
            </ListItem>
          ))}
        </List>
      ) : (
        <div>Loading...</div>
      )}

      {/* Text bar */}
      <MessageInput
        backendURL={backendURL}
        id={id}
        updateMessages={updateMessages}
      />
    </Box>
  );
}
