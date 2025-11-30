import { Box } from "@mui/material";

export interface UserMessageProps {
  /** Indicates if the user sent the message. */
  isFromUser: boolean;
  /** The content of the message. */
  content: string;
}

/**
 * A user message from a private conversation.
 */
export default function UserMessage({ isFromUser, content }: UserMessageProps) {
  const extraStyles = isFromUser
    ? "ml-auto rounded-br-sm bg-biloba-flower-500"
    : "rounded-bl-sm bg-zinc-500";

  return <Box>{content}</Box>;
}
