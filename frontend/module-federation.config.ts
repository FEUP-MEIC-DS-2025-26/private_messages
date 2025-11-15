import { createModuleFederationConfig } from "@module-federation/rsbuild-plugin";

export default createModuleFederationConfig({
  name: "mf_chat",
  exposes: {
    "./Chat": "./src/components/Chat.tsx",
    "./Inbox": "./src/components/Inbox.tsx",
  },
  shared: {
    react: { singleton: true },
    "react-dom": { singleton: true },
  },
});
