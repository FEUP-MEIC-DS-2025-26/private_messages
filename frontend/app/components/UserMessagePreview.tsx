import Image from 'next/image';

interface UserMessagePreviewProps {
  name: string /** The display name of the user. */;
  profilePictureURL: string /** The URL of the user's profile picture. */;
  unreadMessages: number /** The number of unread messages from the user. */;
  lastMessage: string /** The last message sent by the user */;
}

export default function UserMessagePreview({
  name,
  profilePictureURL,
  unreadMessages,
  lastMessage,
}: UserMessagePreviewProps) {
  const notificationText = unreadMessages > 9 ? '9+' : `${unreadMessages}`;

  return (
    <div className="flex items-center gap-5 w-full px-4 py-6 border-b hover:bg-biloba-flower-500">
      <Image
        className="border-solid rounded-full"
        src={profilePictureURL}
        alt={`${name}'${name.endsWith('s') ? '' : 's'} profile picture`}
        width={56}
        height={56}
      />
      <div className="flex-grow">
        <strong>{name}</strong>
        <p>{lastMessage}</p>
      </div>
      {unreadMessages > 0 && (
        <span className="inline-flex w-5 h-5 items-center justify-center text-xs bg-red-600 rounded-full">
          {notificationText}
        </span>
      )}
    </div>
  );
}
