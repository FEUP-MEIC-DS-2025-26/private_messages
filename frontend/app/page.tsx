"use client";

import UserMessagePreview from './components/UserMessagePreview';
import { useState, useEffect } from 'react';

export default function Home() {
  const data: { name: string, latestMessage: string }[] = [];
  const [conversations, setConversations] = useState(data);
  useEffect(() => {
    async function hello() {
      if (conversations.length == 0) {
        const api_url = "http://localhost:8080/api/chat";
        fetch(`${api_url}/login?username=john`)
        .then(() => fetch(`${api_url}/conversation`)).then(res => res.json()).then(data => Promise.all(
          data.map((convoId: number) => Promise.all([
            fetch(`${api_url}/conversation/${convoId}/peer`).then(res => res.json())
            .then(username => fetch(`${api_url}/user/${username}`)).then(res => res.json()).then(({ name }) => name),
              fetch(`${api_url}/conversation/${convoId}/latest`).then(res => res.json())
            .then(msgId => fetch(`${api_url}/message/${msgId}`)).then(res => res.json()).then(({ content }) => content.msg)
          ]))
        )).then(conversationsData => {
          setConversations([...conversations, ...conversationsData.map(([ name, msg ]) => { return { name: name, latestMessage: msg }; })]);
        })
        .catch(console.error);
      }
    }
    hello();

    const intervalId = setInterval(async () => {
      // TODO
    }, 5000);

    return () => clearInterval(intervalId);
  }, [conversations]);
  return (
    <ul className="flex flex-col *:not-last:border-b">
    { conversations.map(({ name, latestMessage }, index) => (
      <li key={`${name}-${index}`}>
      <UserMessagePreview
      name={name}
      profilePictureURL="https://thispersondoesnotexist.com/"
      unreadMessages={Math.floor(Math.random() * 8) + 1}
      lastMessage={latestMessage}
      lastMessageDate="01/01/1970"
      />
      </li>
    ))}
    </ul>
  );
}
