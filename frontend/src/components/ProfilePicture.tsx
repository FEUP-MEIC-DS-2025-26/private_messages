import { Avatar } from '@mui/material';

interface ProfilePictureProps {
  URL: string /** The URL of the profile picture. */;
  name: string /** The name of the profile picture's user. */;
  size: number /** The width and height (in pixels) of the profile picture. */;
}

/**
 * A user's profile picture.
 */
export default function ProfilePicture({
  URL,
  name,
  size,
}: ProfilePictureProps) {
  return (
    <Avatar
      src={URL}
      alt={`${name}'${name.endsWith('s') ? '' : 's'} profile picture`}
      sx={{ width: size, height: size }}
    />
  );
}
