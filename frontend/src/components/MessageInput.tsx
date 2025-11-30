"use client";

import EmojiPicker, { EmojiClickData } from "emoji-picker-react";
import { useRef, useState } from "react";
import { useSWRConfig } from "swr";

// icons
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faPaperPlane, faSmile } from "@fortawesome/free-solid-svg-icons";
import { Box, IconButton, InputAdornment, TextField } from "@mui/material";

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
      await fetch(`${backendURL}/api/chat/conversation/${id}/message`, {
        method: "POST",
        body: new URLSearchParams({ message }),
        headers: { "Content-Type": "application/x-www-form-urlencoded" },
        credentials: "include",
      });

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
    <Box sx={{ position: "relative" }}>
      {/* input field */}
      <TextField
        fullWidth
        size="small"
        placeholder="Type your message here"
        slotProps={{
          input: {
            endAdornment: (
              <InputAdornment position="end">
                {/** send button */}
                <IconButton size="small">
                  <FontAwesomeIcon icon={faPaperPlane} />
                </IconButton>
                {/** button to toggle emoji picker */}
                <IconButton
                  size="small"
                  onClick={() => setShowEmojis(!showEmojis)}
                >
                  <FontAwesomeIcon icon={faSmile} />
                </IconButton>
              </InputAdornment>
            ),
          },
        }}
      />
    </Box>
  );
}
