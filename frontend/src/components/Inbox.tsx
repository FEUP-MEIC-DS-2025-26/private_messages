import useSWR from "swr";
import ChatPreview, { ChatPreviewProps } from "./ChatPreview";

/**
 * A function for fetching the user's conversations from the server.
 * @param {string} URL - the URL
 * @param {string} username - the user's username
 * @returns the user's conversations
 */
const getChats = async (URL: string, username: string) => {
  // login
  await fetch(`${URL}/login?username=${username}`, {
    credentials: "include",
  }).then(console.log);

  // fetch the conversations
  const conversationIDs: number[] = await fetch(`${URL}/conversation`, {
    credentials: "include",
  }).then((res) => res.json());

  // fetch the usernames of the peers with whom we are conversing
  const usernames: string[] = await Promise.all(
    conversationIDs.map((id: number) =>
      fetch(`${URL}/conversation/${id}/peer`, { credentials: "include" }).then(
        (res) => res.json()
      )
    )
  );

  // fetch the peers' display names
  const fullNames: string[] = await Promise.all(
    usernames.map((username: string) =>
      fetch(`${URL}/user/${username}`)
        .then((res) => res.json())
        .then((user) => user.name)
    )
  );

  // fetch the last message from each conversation
  const lastMessages: string[] = await Promise.all(
    conversationIDs.map((id: number) =>
      fetch(`${URL}/conversation/${id}/latest`, { credentials: "include" })
        .then((res) => res.json())
        .then((id: number) =>
          fetch(`${URL}/message/${id}`, { credentials: "include" })
        )
        .then((res) => res.json())
        .then((message) => message.content.msg)
    )
  );

  const products: string[] = await Promise.all(
    conversationIDs.map((id: number) =>
      fetch(`${URL}/conversation/${id}/product`, { credentials: "include" })
        .then((res) => res.json())
        .then((productId: number) =>
          fetch(`${URL}/product/${productId}`, { credentials: "include" })
        )
        .then((res) => res.json())
        .then((product) => product.name)
    )
  );

  // create an array with the conversations
  return conversationIDs.map((id: number, index: number) => ({
    id,
    username: usernames[index],
    name: fullNames[index],
    lastMessage: lastMessages[index],
    profilePictureURL: "https://thispersondoesnotexist.com/",
    unreadMessages: Math.floor(Math.random() * 10),
    product: products[index],
  }));
};

interface InboxProps {
  /** The URL that points to the backend. */
  backendURL: string;
  /** The user's username. */
  username: string;
  /**
   * A function for navigating to a chat.
   * @param {number} id - the unique chat identifier
   */
  goToChat: (id: number) => void;
}

/**
 * The user's inbox.
 */
export default function Inbox({ backendURL, username, goToChat }: InboxProps) {
  const { data: chats, isLoading } = useSWR(
    `${backendURL}/api/chat/conversation`,
    () => getChats(`${backendURL}/api/chat`, username)
  );
  // const chats = await getChats(`${backendURL}/api/chat`, username);

  console.log(chats);
  if (isLoading || !chats) {
    return <div>Loading...</div>;
  }

  return (
    <ul className="flex flex-col overflow-scroll *:not-last:border-b">
      {chats.map((chat: ChatPreviewProps) => (
        <li key={`chat-${chat.id}`}>
          <button
            className="p-0 m-0 w-full text-start"
            onClick={() => goToChat(chat.id)}
          >
            <ChatPreview {...chat} />
          </button>
        </li>
      ))}
    </ul>
  );
}
