import { useState } from "react";

// components
import Inbox from "./components/Inbox";
import Chat from "./components/Chat";

// hard-coded user (only for the prototype)
const USERNAME = "john";

/**
 * The user's inbox.
 */
export default function App() {
  const backendURL =
    import.meta.env.PUBLIC_BACKEND_URL ?? "http://localhost:8080";
  const [chatID, setChatID] = useState<number | null>(null);

  return (
    <div style={{ height: "100vh" }}>
      {chatID ? (
        <Chat
          backendURL={backendURL}
          username={USERNAME}
          id={chatID}
          goToInbox={() => setChatID(null)}
        />
      ) : (
        <Inbox
          backendURL={backendURL}
          username={USERNAME}
          goToChat={(id) => setChatID(id)}
        />
      )}
    </div>
  );
}
