import { createModuleFederationConfig } from '@module-federation/rsbuild-plugin';

export default createModuleFederationConfig({
  name: 'mf_chat',
  exposes: {
    './Chat': './src/components/Chat.tsx',
    './Inbox': './src/components/Inbox.tsx',
  },
  shared: {
    react: {
      singleton: true,
      requiredVersion: '^18.0.0',
    },
    'react-dom': {
      singleton: true,
      requiredVersion: '^18.0.0',
    },
  },
});
