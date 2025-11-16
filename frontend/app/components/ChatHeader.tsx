'use client';

import useSWR from 'swr';

// icons
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faArrowLeft } from '@fortawesome/free-solid-svg-icons';

// components
import ProfilePicture from '@/app/components/ProfilePicture';

/**
 * A function for fetching data from the backend.
 * @param {string} URL - the URL
 */
const fetcher = (URL: string) => fetch(URL).then((res) => res.json());

/**
 * Fetches information regarding the peer.
 * @param {number} id - the chat ID
 */
const getPeer = async (id: number) => {
  const API_URL = '/api/chat';

  // fetch the peer's username
  const username: string = await fetcher(`${API_URL}/conversation/${id}/peer`);

  // fetch the peer's information
  return await fetcher(`${API_URL}/user/${username}`);
};

interface ChatHeaderProps {
  /** The unique chat identifier. */
  id: number;
  /** A function for navigating to the inbox. */
  goToInbox: () => void;
}

/**
 * The header of the chat, which displays information about the peer.
 */
export default function ChatHeader({ id, goToInbox }: ChatHeaderProps) {
  const { data: peer } = useSWR(`/api/chat/conversation/${id}/peer`, () =>
    getPeer(id),
  );

  return (
    <header className="flex items-center gap-5 pl-4 pb-4 border-b">
      <button
        className="inline-flex items-center justify-center w-8 h-8 hover:bg-gray-600 hover:rounded-full transition-all"
        onClick={goToInbox}
      >
        <FontAwesomeIcon icon={faArrowLeft} />
      </button>
      {peer ? (
        <>
          <ProfilePicture
            name={peer.name}
            URL="https://thispersondoesnotexist.com/"
            size={56}
          />
          <div>
            <strong className="text-xl">{peer.name}</strong>
            <p className="text-xs italic before:content-['@']">
              {peer.username}
            </p>
          </div>
        </>
      ) : (
        <div>Loading...</div>
      )}
    </header>
  );
}
