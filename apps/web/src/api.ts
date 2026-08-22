import { client } from '@voxnexus/api-client/client';

/**
 * Same-origin session cookies for every generated API call.
 * Import this module once at startup (see `main.tsx`).
 */
client.setConfig({
  credentials: 'include',
});

export { client };
