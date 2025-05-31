// Nuxt server plugin to initialize Socket.io on server start
import { defineNitroPlugin } from 'nitropack/runtime';
import { setupSocketIO } from '../socket';

// Use 'any' for nitroApp to avoid type mismatch between NitroApp and Nitro
export default defineNitroPlugin((nitroApp: any) => {
  setupSocketIO(nitroApp);
}); 