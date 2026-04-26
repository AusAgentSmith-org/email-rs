import type { CapacitorConfig } from '@capacitor/cli';

const config: CapacitorConfig = {
  appId: 'net.indexarr.rsmail',
  appName: 'rsMail',
  webDir: 'dist',
  server: {
    // Point at the local dev server during development.
    // Remove this block for production builds.
    url: 'http://localhost:8585',
    cleartext: true,
  },
};

export default config;
