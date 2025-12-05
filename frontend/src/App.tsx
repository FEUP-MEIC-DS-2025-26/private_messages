import { Routes, Route } from 'react-router-dom';

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

  return (
    <Routes>
      <Route
        index
        element={<Inbox backendURL={backendURL} userID={USER_ID} />}
      />
      <Route
        path=":id"
        element={<Chat backendURL={backendURL} userID={USER_ID} />}
      />
    </Routes>
  );
}
