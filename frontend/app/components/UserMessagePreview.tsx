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
  return (
    <div className="flex items-center gap-3 w-full p-3 bg-[#107ab0] rounded-xl">
      <Image
        className="border-solid rounded-full"
        src={profilePictureURL}
        alt={`${name}'${name.endsWith('s') ? '' : 's'} profile picture`}
        width={56}
        height={56}
      />
      <div>
        <strong>{name}</strong>
        <p>{lastMessage}</p>
      </div>
    </div>
  );
}
