import Form from 'next/form';

// icons
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faArrowLeft, faPaperPlane } from '@fortawesome/free-solid-svg-icons';

// assets
import ProfilePicture from '../components/ProfilePicture';
import UserMessage from './components/UserMessage';

const mockUser = {
  name: 'John Doe',
  profilePictureURL: 'https://thispersondoesnotexist.com/',
};

export default function Chat() {
  return (
    <>
      <header className="flex items-center gap-5 mb-6 pl-4 pb-4 border-b">
        <a
          className="inline-flex items-center justify-center w-8 h-8 hover:bg-gray-600 hover:rounded-full transition-all"
          href="/"
        >
          <FontAwesomeIcon icon={faArrowLeft} />
        </a>
        <ProfilePicture
          name={mockUser.name}
          URL={mockUser.profilePictureURL}
          size={56}
        />
        <strong className="text-xl">{mockUser.name}</strong>
      </header>

      {/** Chat */}
      <ul className="grow overflow-scroll flex flex-col gap-3 px-3 mb-6">
        <li>
          <UserMessage
            isFromUser={false}
            content="Boa tarde, as laranjas ainda estão à venda?"
          />
        </li>

        <li>
          <UserMessage
            isFromUser={true}
            content="Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum."
          />
        </li>

        <li>
          <UserMessage
            isFromUser={true}
            content="Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum."
          />
        </li>

        <li>
          <UserMessage
            isFromUser={true}
            content="Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum."
          />
        </li>

        <li>
          <UserMessage
            isFromUser={false}
            content="Boa tarde, as laranjas ainda estão à venda?"
          />
        </li>
      </ul>

      {/* Text bar */}
      <Form
        action=""
        className="sticky flex items-center gap-2 px-6 py-1 rounded-full border"
      >
        <input
          className="grow"
          name="message"
          type="text"
          placeholder="Type your message here"
        />
        <button className="w-6 h-6 hover:bg-biloba-flower-800 hover:rounded-full transition-all">
          <FontAwesomeIcon
            className="text-biloba-flower-500 cursor-pointer"
            icon={faPaperPlane}
          />
        </button>
      </Form>
    </>
  );
}
