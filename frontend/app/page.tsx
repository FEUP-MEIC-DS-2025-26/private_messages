"use client";

import UserMessagePreview from './components/UserMessagePreview';
import { useState, useEffect } from 'react';

export default function Home() {
    const [conversations, setConversations] = useState([]);
    useEffect(() => {
        const api_url = "http://localhost:8080/api/chat/conversation";
        setTimeout(async () => {
            fetch(api_url).then(res => res.json()).then(data => {
                // TODO
            });
        }, 1);

        const intervalId = setInterval(async () => {
            // TODO
        }, 5000);

        return () => clearInterval(intervalId);
    }, [conversations]);
  return (
    <ul className="flex flex-col *:not-last:border-b">
      <li>
        <UserMessagePreview
          name="John Doe"
          profilePictureURL="https://thispersondoesnotexist.com/"
          unreadMessages={2}
          lastMessage="Boa tarde, as laranjas ainda estão à venda?"
          lastMessageDate="21/05/2026"
        />
      </li>

      <li>
        <UserMessagePreview
          name="John Doe"
          profilePictureURL="https://thispersondoesnotexist.com/"
          unreadMessages={0}
          lastMessage="Boa tarde, as laranjas ainda estão à venda?"
          lastMessageDate="21/05/2026"
        />
      </li>

      <li>
        <UserMessagePreview
          name="John Doe"
          profilePictureURL="https://thispersondoesnotexist.com/"
          unreadMessages={12}
          lastMessage="Boa tarde, as laranjas ainda estão à venda?"
          lastMessageDate="21/05/2026"
        />
      </li>
    </ul>
  );
}
