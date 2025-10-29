'use client';

import Form from 'next/form';
import EmojiPicker from 'emoji-picker-react';
import { useState } from 'react';

// icons
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faPaperPlane, faSmile } from '@fortawesome/free-solid-svg-icons';

export default function MessageInput() {
  const [message, setMessage] = useState('');

  return (
    <Form
      action=""
      className="sticky flex items-center gap-2 px-6 py-1 rounded-full border-2 focus-within:border-biloba-flower-500 transition-all"
    >
      {/** Input area */}
      <input
        className="grow focus:outline-none focus:caret-biloba-flower-500 focus:caret-2"
        name="message"
        type="text"
        placeholder="Type your message here"
        value={message}
        onChange={(event) => setMessage(event.target.value)}
        required
      />

      {/** Button to toggle the emoji picker */}
      <button
        className="cursor-pointer hover:text-biloba-flower-500 transition-color"
        type="button"
        popoverTarget="emoji-picker"
        popoverTargetAction="toggle"
      >
        <FontAwesomeIcon icon={faSmile} />
      </button>

      <div id="emoji-picker" popover="auto">
        <EmojiPicker
          onEmojiClick={(emoji) => setMessage(message + emoji.emoji)}
        />
      </div>

      {/** Send button */}
      <button className="w-6 h-6 cursor-pointer hover:text-biloba-flower-500 transition-color">
        <FontAwesomeIcon icon={faPaperPlane} />
      </button>
    </Form>
  );
}
