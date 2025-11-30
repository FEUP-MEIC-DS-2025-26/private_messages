"use client";

import EmojiPicker, { EmojiClickData } from "emoji-picker-react";
import { useRef, useState } from "react";
import { useSWRConfig } from "swr";

// icons
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faPaperPlane, faSmile } from "@fortawesome/free-solid-svg-icons";
import {
  Box,
  IconButton,
  InputAdornment,
  Popover,
  TextField,
} from "@mui/material";

interface MessageInputProps {
  /** The URL that points to the backend. */
  backendURL: string;
  /** The unique chat identifier. */
  id: number;
}

/**
 * The input field for sending messages
 */
export default function MessageInput({ backendURL, id }: MessageInputProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  //const [showEmojis, setShowEmojis] = useState(false);

  // to force SWR to refetch the messages
  const { mutate } = useSWRConfig();

  /**
   * A function for sending messages.
   * @param {FormData} data - the form data containing the message
   */
  const sendMessage = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();

    // fetch the form component
    const form = event.currentTarget as HTMLFormElement;
    const formData = new FormData(form);

    // fetch the message
    const message = formData.get("message") as string;

    // if a message exists, send it
    if (message) {
      await fetch(`${backendURL}/api/chat/conversation/${id}/message`, {
        method: "POST",
        body: new URLSearchParams({ message }),
        headers: { "Content-Type": "application/x-www-form-urlencoded" },
        credentials: "include",
      });

      // refetch the messages
      mutate(`/api/chat/conversation/${id}`);
    }

    form.reset();
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
    <Box component="form" onSubmit={sendMessage}>
      {/* input field */}
      <TextField
        fullWidth
        name="message"
        size="small"
        placeholder="Type your message here"
        slotProps={{
          input: {
            endAdornment: (
              <InputAdornment position="end">
                {/** send button */}
                <IconButton size="small" type="submit">
                  <FontAwesomeIcon icon={faPaperPlane} />
                </IconButton>
                {/** button to toggle emoji picker */}
                <IconButton size="small">
                  <FontAwesomeIcon icon={faSmile} />
                </IconButton>
              </InputAdornment>
            ),
          },
        }}
      />

      {/* emoji picker */}
    </Box>
  );
}
