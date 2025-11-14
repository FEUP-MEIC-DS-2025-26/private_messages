import { createModuleFederationConfig } from "@module-federation/rsbuild-plugin";

export default createModuleFederationConfig({
  name: "mip_chat",
  exposes: {
    "./Chat": "./src/components/Chat.tsx",
    "./ChatHeader": "./src/components/ChatHeader.tsx",
    "./ChatPreview": "./src/components/ChatPreview.tsx",
    "./Inbox": "./src/components/Inbox.tsx",
    "./MessageInput": "./src/components/MessageInput.tsx",
    "./ProfilePicture": "./src/components/ProfilePicture.tsx",
    "./UserMessage": "./src/components/UserMessage.tsx",
  },
  shared: {
    react: {
      singleton: true,
      requiredVersion: "^18.0.0",
      eager: true,
    },
    "react-dom": {
      singleton: true,
      requiredVersion: "^18.0.0",
      eager: true,
    },
    "@emotion/react": {
      singleton: true,
      requiredVersion: "^11.0.0",
      eager: true,
    },
  },
});
