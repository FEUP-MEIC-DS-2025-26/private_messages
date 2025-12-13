import { Box, Divider, IconButton, Typography } from '@mui/material';
import { Link } from 'react-router-dom';
import useSWR from 'swr';

// icons
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faArrowLeft } from '@fortawesome/free-solid-svg-icons';

// components
import ProfilePicture from './ProfilePicture';
import { fetcher } from '../utils';

/**
 * Fetches information regarding the peer.
 * @param {number} id - the unique chat identifier
 */
const getPeer = async (id: string, backendURL: string) => {
  // fetch the peer's JumpSeller ID
  const userID: number = await fetcher(
    `${backendURL}/api/chat/conversation/${id}/peer`,
  ).then((peer) => peer.id);

  // fetch the peer's information
  return await fetcher(`${backendURL}/api/chat/user/${userID}`);
};

/**
 * Fetches information regarding the product the chat concerns.
 * @param id - The unique chat identifier
 * @param backendURL - The URL that points to the backend server.
 */
const getProduct = async (id: string, backendURL: string) => {
  return await fetcher(`${backendURL}/api/chat/conversation/${id}/product`)
    .then((x) => x.id)
    .then((productId: number) =>
      fetcher(`${backendURL}/api/chat/product/${productId}`),
    )
    .then((product) => product.name);
};

interface ChatHeaderProps {
  /** The URL that points to the backend server. */
  backendURL: string;
  /** The unique chat identifier. */
  id: string;
}

/**
 * The header of the chat, which displays information about the peer.
 */
export default function ChatHeader({ backendURL, id }: ChatHeaderProps) {
  const { data: peer } = useSWR(
    `${backendURL}/api/chat/conversation/${id}/peer`,
    () => getPeer(id, backendURL),
  );

  const { data: product } = useSWR(
    `${backendURL}/api/chat/conversation/${id}/product`,
    () => getProduct(id, backendURL),
  );

  return (
    <Box
      sx={{
        display: 'flex',
        alignItems: 'center',
        gap: '16px',
        px: '16px',
        py: '16px',
      }}
    >
      {/** button to navigate to the inbox */}
      <IconButton component={Link} to=".." size="small" color="primary">
        <FontAwesomeIcon icon={faArrowLeft} />
      </IconButton>

      {/** peer information */}
      {peer ? (
        <>
          <ProfilePicture
            name={peer.name}
            URL="https://thispersondoesnotexist.com/"
            size={56}
          />
          <Box>
            <Box display="flex" alignItems="center" gap={1}>
              {/* display name */}
              <Typography variant="body1" component="strong" fontWeight="bold">
                {peer.name}
              </Typography>
              <Divider orientation="vertical" flexItem />

              {/* product */}
              <Typography variant="body1" component="span">
                {product}
              </Typography>
            </Box>
            {/* username */}
            <Typography
              variant="body2"
              fontStyle="italic"
              sx={{
                '&::before': {
                  content: '"@"',
                },
              }}
            >
              {peer.username}
            </Typography>
          </Box>
        </>
      ) : (
        <div>Loading...</div>
      )}
    </Box>
  );
}
