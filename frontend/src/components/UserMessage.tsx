import { Box, Typography } from '@mui/material';

// utility
import { formatDate } from '../utils';

export interface UserMessageProps {
  isFromUser: boolean;
  content: string;
  timestamp: Date;
  visible: boolean;
}

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
