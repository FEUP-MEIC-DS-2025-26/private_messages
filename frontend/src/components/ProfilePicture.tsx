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
    <img
      className="border-solid rounded-full"
      src={URL}
      alt={`${name}'${name.endsWith("s") ? "" : "s"} profile picture`}
      width={size}
      height={size}
    />
  );
}
