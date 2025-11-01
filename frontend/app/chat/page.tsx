"use client";

import Link from 'next/link';

// components
import ProfilePicture from '../components/ProfilePicture';
import UserMessage from './components/UserMessage';
import MessageInput from './components/MessageInput';
import { useState, useEffect } from 'react';

// icons
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faArrowLeft } from '@fortawesome/free-solid-svg-icons';

const mockUser = {
  name: 'John Doe',
  profilePictureURL: 'https://thispersondoesnotexist.com/',
};

type Msg = {
    isFromUser: boolean,
    content: string
};

export default function Chat() {
    const state: Msg[] = [];
    const [messages, setMessages] = useState(state);
    const api_url = "http://localhost:8080/api/chat/conversation/1/recent";
    const user_id: number = 1;
    let previous_message_id: number | null = null;

    useEffect(() => {
        const intervalId = setInterval(async () => {
            if (messages.length == 0) {
                fetch(api_url).then(res => res.json()).then(data => {
                    if (data.length != 2) {
                        console.error(`JSON received has wrong length: ${data.length}`);
                    } else {
                        setMessages(data[0].map((record: [uid: number, content: string]) => { return { isFromUser: record[0] === user_id, content: record[1] };}));
                        previous_message_id = data[1];
                    }
                });
            } else {
                console.log("I am making a request");
                setMessages([...messages, {
                    isFromUser: Math.random() > 0.5,
                    content: Math.floor(Math.random() * 1000000).toString()
                }]);
            }
        }, 5000);
        return () => clearInterval(intervalId);
    }, [messages]);
  return (
    <>
      <header className="flex items-center gap-5 pl-4 pb-4 border-b">
        <Link
          className="inline-flex items-center justify-center w-8 h-8 hover:bg-gray-600 hover:rounded-full transition-all"
          href="/"
        >
          <FontAwesomeIcon icon={faArrowLeft} />
        </Link>
        <ProfilePicture
          name={mockUser.name}
          URL={mockUser.profilePictureURL}
          size={56}
        />
        <strong className="text-xl">{mockUser.name}</strong>
      </header>

      {/** Chat */}
      <ul className="grow overflow-scroll flex flex-col gap-3 px-3">
        { messages.map(({ isFromUser, content }, index) => (
            <li key={`${content}-${index}`}>
                <UserMessage
                    isFromUser={isFromUser}
                    content={content}
                />
            </li>
            ))
        }
        </ul>
      {/* Text bar */}
      <MessageInput />
    </>
  );
}
