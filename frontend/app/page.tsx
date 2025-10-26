import UserMessagePreview from './components/UserMessagePreview';

export default function Home() {
  return (
    <main className="flex flex-col m-20 justify-center">
      <UserMessagePreview
        name="John Doe"
        profilePictureURL="https://thispersondoesnotexist.com/"
        unreadMessages={2}
        lastMessage="Boa tarde, as laranjas ainda estão à venda?"
      />
      <UserMessagePreview
        name="John Doe"
        profilePictureURL="https://thispersondoesnotexist.com/"
        unreadMessages={0}
        lastMessage="Boa tarde, as laranjas ainda estão à venda?"
      />
      <UserMessagePreview
        name="John Doe"
        profilePictureURL="https://thispersondoesnotexist.com/"
        unreadMessages={12}
        lastMessage="Boa tarde, as laranjas ainda estão à venda?"
      />
    </main>
  );
}
