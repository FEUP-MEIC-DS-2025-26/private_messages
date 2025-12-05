import { Routes, Route } from 'react-router-dom';
import { Box } from '@mui/material';

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
    <Box sx={{ height: '80vh' }}>
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
    </Box>
  );
}
