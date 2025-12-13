import { Box } from '@mui/material';
import { Ref, useLayoutEffect, useRef, useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import SearchBar from './SearchBar';

// components
import ChatHeader from './ChatHeader';
import UserMessage, { UserMessageProps } from './UserMessage';
import MessageInput from './MessageInput';
import { Divider, List, ListItem } from '@mui/material';
import { fetcher, login } from '../utils';

interface ChatProps {
  backendURL: string;
  id: number;
  userID: number;
  goToInbox: () => void;
}

async function getMessages(
  URL: string,
  userID: number,
): Promise<UserMessageProps[]> {
  const messages: any[] = await fetcher(`${URL}/recent`).then(
    ({ content }) => content,
  );

  return messages.map((message) => ({
    isFromUser: message.sender_jsid === userID,
    content: message.msg.contents,
    timestamp: new Date(message.msg.timestamp),
    visible: true,
  }));
}

async function pollMessages(
  backendURL: string,
  userID: number,
  stateMessageId: number,
  latestMessageId: number,
): Promise<UserMessageProps[]> {
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
      visible: true,
    });
  }

  return newMessages;
}

export default function Chat({ backendURL, id, userID, goToInbox }: ChatProps) {
  const url = `${backendURL}/api/chat/conversation/${id}`;

  if (!id) {
    return <div>Loading...</div>;
  }

  // variables for the messages
  const messageListRef: Ref<HTMLUListElement> = useRef(null);
  const [messageId, setMessageId] = useState<number>(-1);
  const [messages, setMessages] = useState<UserMessageProps[]>([]);

  /*
  // For some reason, these two need to be in the same function
  useSWR(url, async (URL: string) => {
    const messageId: number = await fetcher(`${URL}/latest`).then((x) => x.id);
    setMessageId(messageId);

    const msgs: UserMessageProps[] = await getMessages(URL, userID);
    setMessages(msgs);
  });
  */

  // This prevents the user from scrolling up
  /*
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
  */

  try {
    if (messageId == -1) {
      fetcher(`${url}/latest`).then(({ id }) => {
        setMessageId(id);
      });
    }

    if (messages.length == 0) {
      getMessages(url, userID).then((msgs) => {
        setMessages(msgs);
      });
    }

    useEffect(() => {
      const intervalId = setInterval(async () => {
        // msgId is only -1 when the data hasn't been fetched yet. No matter the result, msgId cannot be -1 afterwards
        try {
          if (messageId != -1) {
            const latestMessageId: number = await fetcher(
              `${backendURL}/api/chat/conversation/${id}/latest`,
            ).then(({ id }) => id);
            if (latestMessageId !== null && latestMessageId !== undefined) {
              const newMessages: UserMessageProps[] = await pollMessages(
                backendURL,
                userID,
                messageId,
                latestMessageId,
              );

              if (newMessages && newMessages.length > 0) {
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
        } catch (e) {
          await login(`${backendURL}/api/chat`, userID);
        }
      }, 5000);
      return () => clearInterval(intervalId);
    }, [messages]);
  } catch (e) {
    // login
    login(`${backendURL}/api/chat`, userID);
  }

  const updateMessages = async (latestMessageId: number) => {
    const newMessages: UserMessageProps[] = await pollMessages(
      backendURL,
      userID,
      messageId,
      latestMessageId,
    );
    if (newMessages.length > 0) {
      newMessages.reverse();
      setMessageId(latestMessageId);
      setMessages([...messages, ...newMessages]);
    }
  };

  const filter = (text: string) => {
    setMessages(
      messages.map((message) => ({
        ...message,
        visible: message.content.toLowerCase().includes(text.toLowerCase()),
      })),
    );
  };

  return (
    <Box
      sx={{
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
      }}
    >
      <ChatHeader backendURL={backendURL} id={id} goToInbox={goToInbox} />
      <Divider />
      <SearchBar filter={filter} />
      <Divider />
      {messages ? (
        <List
          ref={messageListRef}
          sx={{
            flexGrow: 1,
            display: 'flex',
            flexDirection: 'column',
            py: '16px',
            maxHeight: '100%',
            gap: '8px',
            overflow: 'auto',
          }}
        >
          {messages
            .filter(({ visible }) => visible)
            .map((message: UserMessageProps, index: number) => (
              <ListItem key={`message-${index}`} sx={{ py: 0 }}>
                <UserMessage {...message} />
              </ListItem>
            ))}
        </List>
      ) : (
        <div>Loading...</div>
      )}

      {/* text bar */}
      <MessageInput
        backendURL={backendURL}
        id={id}
        updateMessages={updateMessages}
      />
    </Box>
  );
}
