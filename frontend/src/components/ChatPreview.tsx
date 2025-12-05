import { Badge, Box, Divider, ListItemButton, Typography } from '@mui/material';

// components
import ProfilePicture from './ProfilePicture';

interface ChatPreviewProps {
  id: number;
  name: string;
  username: string;
  profilePictureURL: string;
  unreadMessages: number;
  lastMessage: string;
  product: string;
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

      <Box>
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
        {/** last message */}
        <Typography variant="body2">{lastMessage}</Typography>
      </Box>
    </ListItemButton>
  );
}
