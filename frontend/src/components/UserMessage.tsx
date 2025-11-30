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
    ? {
        backgroundColor: "primary.main",
        borderBottomRightRadius: 2,
        ml: "auto",
      }
    : { backgroundColor: "grey.400", borderBottomLeftRadius: 2 };

  return (
    <Box
      sx={{
        padding: "12px",
        maxWidth: "75%",
        borderRadius: 6,
        color: "text.primary",
        ...extraStyles,
      }}
    >
      {content}
    </Box>
  );
}
