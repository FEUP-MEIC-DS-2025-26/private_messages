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
        <ProfilePicture
          name={mockUser.name}
          URL={mockUser.profilePictureURL}
          size={56}
        />
        <strong className="text-xl">{mockUser.name}</strong>
      </header>

      {/** Chat */}
      <ul className="flex flex-col gap-3">
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
      </ul>
    </>
  );
}
