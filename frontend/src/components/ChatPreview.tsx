import { Badge, Box, Divider, ListItemButton, Typography } from '@mui/material';

// components
import ProfilePicture from './ProfilePicture';
import { UserMessageProps } from './UserMessage';
import { formatDate } from '../utils';

export interface ChatPreviewProps {
  id: number;
  userID: number;
  name: string;
  username: string;
  lastMessage: UserMessageProps;
  profilePictureURL: string;
  unreadMessages: number;
  product: string;
  visible: boolean;
}

export default function ChatPreview({
  name,
  username,
  profilePictureURL,
  unreadMessages,
  lastMessage,
  product,
}: ChatPreviewProps) {
  return (
    <ListItemButton
      sx={{
        display: 'flex',
        alignItems: 'center',
        gap: '20px',
        width: 1,
        py: '16px',
      }}
    >
      {/** profile picture with notification counter */}
      <Badge
        badgeContent={unreadMessages}
        max={9}
        color="primary"
        overlap="circular"
      >
        <ProfilePicture name={name} URL={profilePictureURL} size={56} />
      </Badge>

      <Box sx={{ minWidth: 0, flexGrow: 1 }}>
        <Box sx={{ display: 'flex', gap: 1, alignItems: 'center' }}>
          {/** display name */}
          <Typography component="strong" variant="body1" fontWeight="bold">
            {name}
          </Typography>
          {/** username */}
          <Typography
            component="span"
            variant="body2"
            display="inline"
            fontStyle="italic"
            sx={{
              '&::before': {
                content: '"@"',
              },
            }}
          >
            {username}
          </Typography>
          {/** product */}
          <Divider orientation="vertical" flexItem aria-hidden />
          <Typography component="span" variant="body2">
            {product}
          </Typography>
        </Box>
        <Box
          sx={{
            display: 'flex',
            gap: 3,
            alignItems: 'center',
            justifyContent: 'space-between',
          }}
        >
          {/** last message */}
          <Typography
            variant="body2"
            sx={{
              flexGrow: 1,
              whiteSpace: 'nowrap',
              overflow: 'hidden',
              textOverflow: 'ellipsis',
              fontStyle: lastMessage.isFromUser ? 'italic' : 'normal',
            }}
          >
            {lastMessage.content}
          </Typography>
          {/** last message timestamp */}
          <Typography
            variant="caption"
            sx={{
              flexShrink: 0,
              whiteSpace: 'nowrap',
              overflow: 'hidden',
              textOverflow: 'ellipsis',
            }}
          >
            {formatDate(lastMessage.timestamp)}
          </Typography>
        </Box>
      </Box>
    </ListItemButton>
  );
}
