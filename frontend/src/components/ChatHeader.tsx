"use client";

import useSWR from "swr";

// icons
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faArrowLeft } from "@fortawesome/free-solid-svg-icons";

// components
import ProfilePicture from "./ProfilePicture";

/**
 * A function for fetching data from the backend.
 * @param {string} URL - the URL
 */
const fetcher = (URL: string) => fetch(URL, {credentials: "include"}).then((res) => res.json());

/**
 * Fetches information regarding the peer.
 * @param {number} id - the chat ID
 */
const getPeer = async (id: number, backendURL: string) => {
  // fetch the peer's username
  const username: string = await fetcher(`${backendURL}/conversation/${id}/peer`);

  // fetch the peer's information
  return await fetcher(`${backendURL}/user/${username}`);
};

const getProduct = async (id: number, backendURL: string) => {
  return await fetch(`${backendURL}/conversation/${id}/product`, {credentials: "include"})
    .then(res => res.json())
    .then((productId: number) => fetch(`${backendURL}/product/${productId}`, {credentials: "include"}))
    .then(res => res.json())
    .then(product => product.name);
}

interface ChatHeaderProps {
  /** The URL that points to the backend server. */
  backendURL: string;
  /** The unique chat identifier. */
  id: number;
  /** A function for navigating to the inbox. */
  goToInbox: () => void;
}

/**
 * The header of the chat, which displays information about the peer.
 */
export default function ChatHeader({
  backendURL,
  id,
  goToInbox,
}: ChatHeaderProps) {
  const { data: peer } = useSWR(
    `${backendURL}/api/chat/conversation/${id}/peer`,
    () => getPeer(id, backendURL)
  );
  
  const { data: product } = useSWR(`${backendURL}/api/chat/conversation/${id}/product`, () => getProduct(id));
  
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
            <strong className="text-xl">{peer.name}</strong> | <span className="text-xl">{product}</span>
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
