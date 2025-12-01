import { useState } from 'react';

// components
import Inbox from './components/Inbox';
import Chat from './components/Chat';

// hard-coded user (only for the prototype)
const USER_ID = 1;

/**
 * The user's inbox.
 */
export default function App() {
  const backendURL =
    import.meta.env.PUBLIC_BACKEND_URL ?? 'http://localhost:8080';
  const [chatID, setChatID] = useState<number | null>(null);

  return (
    <div style={{ height: '100vh' }}>
      {chatID ? (
        <Chat
          backendURL={backendURL}
          userID={USER_ID}
          id={chatID}
          goToInbox={() => setChatID(null)}
        />
      ) : (
        <Inbox
          backendURL={backendURL}
          userID={USER_ID}
          goToChat={(id) => setChatID(id)}
        />
      )}
    </div>
  );
}
