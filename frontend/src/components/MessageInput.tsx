"use client";

import EmojiPicker, { EmojiClickData } from "emoji-picker-react";
import { useRef, useState } from "react";
import { useSWRConfig } from "swr";

// icons
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faPaperPlane, faSmile } from "@fortawesome/free-solid-svg-icons";

interface MessageInputProps {
  /** The URL that points to the backend. */
  backendURL: string;
  /** The unique chat identifier. */
  id: number;
}

/**
 * The input field for sending messages
 */
export default function MessageInput({ backendURL, id, updateMessages }: MessageInputProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [showEmojis, setShowEmojis] = useState(false);

  // to force SWR to refetch the messages
  const { mutate } = useSWRConfig();

  /**
   * A function for sending messages.
   * @param {FormData} data - the form data containing the message
   */
  const sendMessage = async (data: FormData) => {
    const message = data.get("message") as string;

    // if a message exists, send it
    if (message) {
      const latestMessageId = await fetch(`${backendURL}/api/chat/conversation/${id}/message`, {
        method: "POST",
        body: new URLSearchParams({ message }),
        headers: { "Content-Type": "application/x-www-form-urlencoded" },
        credentials: "include",
      }).then(res => res.json());

      await updateMessages(latestMessageId);

      // refetch the messages
      mutate(`/api/chat/conversation/${id}`);
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
      "end"
    );
  };

  return (
    <div className="relative">
      <form
        onSubmit={async (e) => {
          e.preventDefault();
          const form = e.currentTarget as HTMLFormElement;
          const formData = new FormData(form);
          await sendMessage(formData);
          form.reset();
        }}
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
          className={`cursor-pointer transition-color ${
            showEmojis ? "" : "hover:"
          }text-biloba-flower-500`}
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
