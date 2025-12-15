import { Routes, Route } from 'react-router-dom';
import { Box } from '@mui/material';
import { useEffect, useState } from 'react';

// components
import Inbox from './components/Inbox';
import Chat from './components/Chat';
import { login, me } from './utils';
import ErrorPage from './components/ErrorPage';
import NewConversation from './components/NewConversation';

// hard-coded user (only for the prototype)
const DEFAULT_USER_ID = 1;

/**
 * The user's inbox.
 */
export default function App() {
  const backendURL =
    import.meta.env.PUBLIC_BACKEND_URL ?? 'http://localhost:8080';
  const [chatID, setChatID] = useState<number | null>(null);
  const [userID, setUserID] = useState<number | null>(null);

  useEffect(() => {
    if (userID == null || userID == -1) {
      const login_fetch_data = async () => {
        try {
          await login(backendURL + '/api/chat', DEFAULT_USER_ID);
          const my_id = await me(backendURL + '/api/chat');
          setUserID(my_id);
        } catch (error) {
          setUserID(-1);
        }
      };
      login_fetch_data();
    }
  }, []);

  if (userID == null) {
    return <div>Loading...</div>;
  }

  if (userID == -1) {
    return (
      <ErrorPage
        message={'The user is not logged in. Please click login in the navbar. Redirecting in 5 seconds...'}
        error={undefined}
        redirectURL={'/auth'}
      />
    );
  }
    
  return (
    <Box sx={{ height: '80vh' }}>
      <Routes>
        <Route
          index
          element={<Inbox backendURL={backendURL} userID={userID} />}
        />
        <Route
          path=":id"
          element={<Chat backendURL={backendURL} userID={userID} />}
        />
        <Route
          path="new"
          element={<NewConversation backendURL={backendURL} userID={userID} />}
        />
      </Routes>
    </Box>
  );
}
