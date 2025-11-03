'use client';

import EmojiPicker, { EmojiClickData } from 'emoji-picker-react';
import { useRef, useState, FormEvent } from 'react';

// icons
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faPaperPlane, faSmile } from '@fortawesome/free-solid-svg-icons';

/**
 * The input field for sending messages
 */
export default function MessageInput({ id }: { id: number }) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [showEmojis, setShowEmojis] = useState(false);

  // a function for sending messages
  const sendMessage = async (data: FormData) => {
    const message = data.get('message') as string;

    // if a message exists, send it
    if (message) {
      await fetch(`/api/chat/conversation/${id}/message`, {
        method: 'POST',
        body: new URLSearchParams({ message }),
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      });
    }
  };

  // a function for handling emoji clicks
  const handleEmojiClick = (emojiData: EmojiClickData) => {
    const input: HTMLInputElement | null = inputRef.current;

    // insert emoji at cursor position
    input?.setRangeText(
      emojiData.emoji,
      input.selectionStart ?? 0,
      input.selectionEnd ?? 0,
      'end',
    );
  };

  return (
    <div className="relative">
      <form
        action={sendMessage}
        className="sticky flex items-center gap-2 px-6 py-1 rounded-full border-2 focus-within:border-biloba-flower-500 transition-all"
      >
        {/** Input area */}
        <input
          ref={inputRef}
          className="grow focus:outline-none focus:caret-biloba-flower-500 focus:caret-2"
          name="message"
          type="text"
          placeholder="Type your message here"
          required
        />

        {/** Button to toggle the emoji picker */}
        <button
          className={`cursor-pointer transition-color ${showEmojis ? '' : 'hover:'}text-biloba-flower-500`}
          type="button"
          onClick={() => setShowEmojis(!showEmojis)}
        >
          <FontAwesomeIcon icon={faSmile} />
        </button>

        {/** Send button */}
        <button
          className="w-6 h-6 cursor-pointer hover:text-biloba-flower-500 transition-color"
          type="submit"
        >
          <FontAwesomeIcon icon={faPaperPlane} />
        </button>
      </form>

      {/* Emoji picker */}
      <div className="absolute right-0 bottom-10 z-10" hidden={!showEmojis}>
        <EmojiPicker onEmojiClick={handleEmojiClick} />
      </div>
    </div>
  );
}
