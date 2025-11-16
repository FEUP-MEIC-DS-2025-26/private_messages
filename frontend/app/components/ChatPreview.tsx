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
}: ChatPreviewProps) {
  const notificationText = unreadMessages > 9 ? '9+' : `${unreadMessages}`;

  return (
    <div className="flex items-center gap-5 w-full px-4 py-6 hover:bg-biloba-flower-500 transition-colors">
      <div className="relative">
        <ProfilePicture name={name} URL={profilePictureURL} size={56} />
        {unreadMessages > 0 && (
          <span className="inline-flex w-5 h-5 items-center justify-center text-xs bg-red-600 rounded-full absolute top-0 right-0">
            {notificationText}
          </span>
        )}
      </div>
      <div>
        <strong>{name}</strong>
        <span className="text-xs ml-2 italic before:content-['@']">
          {username}
        </span>
        <p>{lastMessage}</p>
      </div>
    </div>
  );
}
