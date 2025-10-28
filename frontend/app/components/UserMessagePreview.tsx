import ProfilePicture from './ProfilePicture';

interface UserMessagePreviewProps {
  /** The display name of the user. */
  name: string;
  /** The URL of the user's profile picture. */
  profilePictureURL: string;
  /** The number of unread messages from the user. */
  unreadMessages: number;
  /** The last message sent by the user */
  lastMessage: string;
  /** The date when the last message was sent. */
  lastMessageDate: string;
}

/**
 * A preview of the messages chat with a given user.
 */
export default function UserMessagePreview({
  name,
  profilePictureURL,
  unreadMessages,
  lastMessage,
  lastMessageDate,
}: UserMessagePreviewProps) {
  const notificationText = unreadMessages > 9 ? '9+' : `${unreadMessages}`;

  return (
    <a
      className="flex items-center gap-5 w-full px-4 py-6 border-b hover:bg-biloba-flower-500 transition-colors"
      href="/chat"
    >
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
        <span className="text-xs ml-3">{lastMessageDate}</span>
        <p>{lastMessage}</p>
      </div>
    </a>
  );
}
