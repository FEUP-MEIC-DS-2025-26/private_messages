'use client';

import Link from 'next/link';
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
 * @param {string} id - the chat ID
 */
const getPeer = async (id: string) => {
  const API_URL = '/api/chat';

  // fetch the peer's username
  const username: string = await fetcher(`${API_URL}/conversation/${id}/peer`);

  // fetch the peer's display name
  const name: string = await fetcher(`${API_URL}/user/${username}`);

  return { name, username };
};

/**
 * The header of the chat, which displays information about the peer.
 */
export default function ChatHeader({ id }: { id: number }) {
  const { data: peer, isLoading } = useSWR(
    `/api/chat/conversation/${id}/peer`,
    fetcher,
  );

  return (
    <header className="flex items-center gap-5 pl-4 pb-4 border-b">
      <Link
        className="inline-flex items-center justify-center w-8 h-8 hover:bg-gray-600 hover:rounded-full transition-all"
        href="/chat"
      >
        <FontAwesomeIcon icon={faArrowLeft} />
      </Link>
      {isLoading ? (
        <div>Loading...</div>
      ) : (
        <>
          <ProfilePicture
            name={peer.name}
            URL={peer.profilePictureURL}
            size={56}
          />
          <strong className="text-xl">{peer.name}</strong>
          <p className="text-xs italic before:content-['@']">{peer.username}</p>
        </>
      )}
    </header>
  );
}
