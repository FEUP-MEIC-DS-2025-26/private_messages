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
    const api_url = "http://localhost:8080/api/chat/conversation";
    const user_id: number = 1;

    useEffect(() => {

        const quotes: string[] = [
            "My name is ChatGPT, but you can call me <Uncaught SyntaxError: JSON.parse: unexpected character at line 1 column 1 of the JSON data>",
            "Yeah? yeah yeah? yeah? Yeah? Agile or something. Yeah? yeah yeah yeah yeah yeah yeah yeah yeah?",
            "I live in the shadows. For Scrum!",
            "Scrum Master sounds like a dirty title and nobody will ever change my mind",
            "Weeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
            "My favorite sleeping position is the party escort submission position",
            "Cake and grief counseling will be available at the conclusion of the presentation",
            "I'm a potato",
            "Haha Haha Haha Haha Haha Haha",
            "Do you perhaps have some oranges? I like 'em big, I like 'em chunky",
            "Rust isn't that good, to be honest. It leaves things with a strange color",
            "I don't know what I'm doing, and neither do you",
            "Fire the copilot, emerge the seek, unfrench the claude, obliterate the gemini, keep the fish's stock",
            "I don't want to sound rude, but what even is a sprint backlog? Do people actually run with logs on their back?",
            "If there are no rules without exceptions, then there must be a rule without exceptions, since this would be an exception of the no-exceptions rule. This leads to the fact that this rule enforces exceptions on almost every rule except on itself. Reminds me of someone",
            "Why does Moodle still lack a dark mode?",
            "I'm going to start enforcing developers to use smoke signals to communicate with each other. One of the life hacks I learned recently",
            "My grandma does not use the Internet",
            "Take a deep breath and repeat this mantra: this only ends in June of 2027, hopefully",
            "Maria Albertina, como foste nessa, de chamar Vanessa, à tua menina?",
            "I miss Software Engineering :( ... but my aim is getting better!",
            "I smell cheese",
            "ChatGPT, summarize my students' projects for me",
            "What is algebra? Is it those things with three sides?",
            "Mom, I'm in an university presentation!",
            "AÇORES"
        ];

        async function hello() {
            if (messages.length == 0) {
                fetch(`${api_url}/1/recent`)
                    .then(res => res.json())
                    .then(({ content, previous_msg }) => {
                        setMessages(content.map((record: { sender_id: number, msg: string }) => {
                            return {
                                isFromUser: record.sender_id === user_id,
                                content: record.msg
                            };
                        }));
                        setPreviousMessageId(previous_msg);
                    }
                );
            }
        }
        hello();

        const intervalId = setInterval(async () => {
            if (Math.random() < 0.25) {
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
