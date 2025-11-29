import { Badge } from "@mui/material";

// components
import ProfilePicture from "./ProfilePicture";

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
    <div className="flex items-center gap-5 w-full px-4 py-6 hover:bg-biloba-flower-500 transition-colors">
      <Badge
        badgeContent={unreadMessages}
        max={9}
        color="primary"
        overlap="circular"
      >
        <ProfilePicture name={name} URL={profilePictureURL} size={56} />
      </Badge>
      <div>
        <strong>{name}</strong>
        <span className="text-xs ml-2 italic before:content-['@']">
          {username}
        </span>{" "}
        | <span className="text-xs ml-2">{product}</span>
        <p>{lastMessage}</p>
      </div>
    </div>
  );
}
