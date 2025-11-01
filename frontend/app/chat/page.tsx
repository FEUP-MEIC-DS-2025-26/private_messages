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

export default function Chat() {
    const [messages, setMessages] = useState([{
        isFromUser: true,
        content: "Next.js isn't that good"
    }]);
    useEffect(() => {
        const intervalId = setInterval(async () => {
            console.log("I am making a request");
            setMessages([...messages, {
                isFromUser: Math.random() > 0.5,
                content: Math.floor(Math.random() * 1000000).toString()
            }]);
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
            <li key={`{content}-{index}`}>
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
