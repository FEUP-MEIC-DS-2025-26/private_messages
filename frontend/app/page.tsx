import UserMessagePreview from './components/UserMessagePreview';

export default function Home() {
  return (
    <ul className="flex flex-col *:not-last:border-b">
      <li>
        <UserMessagePreview
          name="John Doe"
          profilePictureURL="https://thispersondoesnotexist.com/"
          unreadMessages={2}
          lastMessage="Boa tarde, as laranjas ainda estão à venda?"
          lastMessageDate="21/05/2026"
        />
      </li>

      <li>
        <UserMessagePreview
          name="John Doe"
          profilePictureURL="https://thispersondoesnotexist.com/"
          unreadMessages={0}
          lastMessage="Boa tarde, as laranjas ainda estão à venda?"
          lastMessageDate="21/05/2026"
        />
      </li>

      <li>
        <UserMessagePreview
          name="John Doe"
          profilePictureURL="https://thispersondoesnotexist.com/"
          unreadMessages={12}
          lastMessage="Boa tarde, as laranjas ainda estão à venda?"
          lastMessageDate="21/05/2026"
        />
      </li>
    </ul>
  );
}
