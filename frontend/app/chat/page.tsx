"use client";

import Link from 'next/link';

// components
import ProfilePicture from '../components/ProfilePicture';
import UserMessage from './components/UserMessage';
import MessageInput from './components/MessageInput';
import { useState, useEffect, useRef, Ref } from 'react';

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
    const [previousMessageId, setPreviousMessageId] = useState(null);
    const ul: Ref<HTMLUListElement> | undefined = useRef(null);
    const api_url = "http://localhost:8080/api/chat/conversation/1/recent";
    const user_id: number = 1;

    useEffect(() => {

        const quotes: string[] = [
            "My name is ChatGPT, but you can call me <Uncaught SyntaxError: JSON.parse: unexpected character at line 1 column 1 of the JSON data>",
            "Yeah? yeah yeah? yeah? Yeah? Agile of something. Yeah? yeah yeah yeah yeah yeah yeah yeah yeah?",
            "I live in the shadows. For Scrum!",
            "a",
            "Scrum Master sounds like a dirty title and nobody will ever change my mind",
            "Weeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
            "My favorite sleeping position is the party escort submission position",
            "UwU",
            "Cake and grief counseling will be available at the conclusion of the test",
            "I'm a potato",
            "*sniff* *sniff*",
            "*woof* *woof",
            "Haha Haha Haha Haha Haha Haha",
        ];

        setTimeout(async () => {
            if (messages.length == 0) {
                fetch(api_url)
                    .then(res => res.json())
                    .then(data => {
                        if (data.length != 2) {
                            console.error(`JSON received has wrong length: ${data.length}`);
                        } else {
                            setMessages(data[0].map((record: [uid: number, content: string]) => {
                                return {
                                    isFromUser: record[0] === user_id,
                                    content: record[1]
                                };
                            }));
                            setPreviousMessageId(data[1]);
                        }
                    }
                );
            }
        }, 1); 

        const intervalId = setInterval(async () => {
            if (Math.random() < 0.15) {
                setMessages([...messages, {
                    isFromUser: Math.random() > 0.5,
                    content: quotes[Math.floor(Math.random() * quotes.length)]
                }]);
            }
            if (ul.current != null && ul.current.lastElementChild != null) {
                ul.current.lastElementChild.scrollIntoView({ behavior: "smooth" });
            }
        }, 500);

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
      <ul className="grow overflow-scroll flex flex-col gap-3 px-3" ref={ul}>
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
