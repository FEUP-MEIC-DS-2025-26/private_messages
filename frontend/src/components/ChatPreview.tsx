import { Badge, Box, Divider, ListItemButton, Typography } from '@mui/material';

// components
import ProfilePicture from './ProfilePicture';

export interface ChatPreviewProps {
  /** The unique identifier of the conversation. */
  id: number;
  /** The display name of the user. */
  name: string;
  /** The user's username. */
  username: string;
  /** The URL of the user's profile picture. */
  profilePictureURL: string;
  /** The number of unread messages from the user. */
  unreadMessages: number;
  /** The last message sent by the user */
  lastMessage: string;
  /** The product the conversation pertains to */
  product: string;
}

/**
 * A preview of the chat with a given user.
 */
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

      <div>
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
      </div>
    </ListItemButton>
  );
}
