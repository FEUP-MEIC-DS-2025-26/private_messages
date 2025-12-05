import { Box, Typography } from '@mui/material';

export interface UserMessageProps {
  isFromUser: boolean;
  content: string;
  timestamp: Date;
  visible: boolean;
}

/**
 * Formats a date as a string.
 * @param date - the date to format
 * @returns a string representing the date
 */
const formatDate = (date: Date): string => {
  const components: string[] = [];

  // compute the days elapsed since the message was sent
  const elapsedDays = (Date.now() - date.getTime()) / 86_400_000;

  if (elapsedDays == 1) {
    components.push('Yesterday');
  } else if (elapsedDays > 1) {
    const day = date.getDate();
    const month = date.getMonth() + 1; // month is 0-based
    const year = date.getFullYear();

    components.push(
      `${String(day).padStart(2, '0')}/${String(month).padStart(2, '0')}/${year}`,
    );
  }

  // format the hour and minutes
  components.push(
    `${date.getHours()}:${String(date.getMinutes()).padStart(2, '0')}`,
  );

  return components.join(' ');
};

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
