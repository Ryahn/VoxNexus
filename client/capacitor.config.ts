import { CapacitorConfig } from '@capacitor/cli';

const config: CapacitorConfig = {
  appId: 'com.voxnexus.app',
  appName: 'VoxNexus',
  webDir: 'dist',
  server: {
    androidScheme: 'https'
  }
};

export default config;
