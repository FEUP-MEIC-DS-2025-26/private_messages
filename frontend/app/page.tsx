import UserMessagePreview from './components/UserMessagePreview';

export default function Home() {
  return (
    <main className="flex m-10 flex-column justify-center">
      <UserMessagePreview
        name="John Doe"
        profilePictureURL="https://thispersondoesnotexist.com/"
        unreadMessages={2}
        lastMessage="Boa tarde, as laranjas ainda estão à venda?"
      />
    </main>
  );
}
