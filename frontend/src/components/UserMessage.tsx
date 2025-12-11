import { Box, Typography } from '@mui/material';

// utility
import { formatDate } from '../utils';

export interface UserMessageProps {
  /** Indicates if the user sent the message. */
  isFromUser: boolean;
  /** The content of the message. */
  content: string;
  /** The date when the message was sent */
  timestamp: Date;
}

/**
 * A user message from a private conversation.
 */
export default function UserMessage({
  isFromUser,
  content,
  timestamp,
}: UserMessageProps) {
  const extraStyles = isFromUser
    ? {
        backgroundColor: 'primary.main',
        borderBottomRightRadius: 2,
        marginLeft: 'auto',
        color: 'primary.contrastText',
      }
    : {
        backgroundColor: 'secondary.main',
        borderBottomLeftRadius: 2,
        color: 'secondary.contrastText',
      };

  return (
    <Box
      sx={{
        display: 'flex',
        flexDirection: 'column',
        gap: 0.5,
        padding: '12px',
        maxWidth: '75%',
        borderRadius: 6,
        ...extraStyles,
      }}
    >
      <Typography variant="body1">{content}</Typography>
      <Typography
        variant="caption"
        sx={{ marginLeft: isFromUser ? 0 : 'auto' }}
      >
        {formatDate(timestamp)}
      </Typography>
    </Box>
  );
}
