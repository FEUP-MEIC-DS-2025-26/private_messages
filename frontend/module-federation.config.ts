import { createModuleFederationConfig } from "@module-federation/rsbuild-plugin";

export default createModuleFederationConfig({
  name: "mip_chat",
  exposes: {
    "./Chat": "./src/components/Chat.tsx",
    "./Inbox": "./src/components/Inbox.tsx",
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
