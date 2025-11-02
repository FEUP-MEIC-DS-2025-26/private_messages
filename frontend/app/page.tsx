'use client';

import { useState } from 'react';

// components
import Inbox from './components/Inbox';
import Chat from './components/Chat';

// hard-coded user (only for the prototype)
const USERNAME = 'john';

/**
 * The user's inbox.
 */
export default function Page() {
  const [chatID, setChatID] = useState<number | null>(null);

  return chatID ? (
    <Chat id={chatID} />
  ) : (
    <Inbox username={USERNAME} setChat={(id) => setChatID(id)} />
  );
}
