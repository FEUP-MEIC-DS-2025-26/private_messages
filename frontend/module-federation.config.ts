import { createModuleFederationConfig } from '@module-federation/rsbuild-plugin';

export default createModuleFederationConfig({
  name: 'mf_chat',
  filename: 'remoteEntry.js',
  exposes: {
    './App': './src/App.tsx',
    './Chat': './src/components/Chat.tsx',
    './Inbox': './src/components/Inbox.tsx',
  },
  shared: {
    '@emotion/react': {
      singleton: true,
      requiredVersion: '^11.0.0',
    },
    react: {
      singleton: true,
      requiredVersion: '^18.0.0',
    },
    'react-dom': {
      singleton: true,
      requiredVersion: '^18.0.0',
    },
    'react-router-dom': {
      singleton: true,
      requiredVersion: '^6.0.0',
    },
  },
});
