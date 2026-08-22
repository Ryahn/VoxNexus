import type { AuthSessionResponse } from '@voxnexus/api-client';
import { createContext, useContext } from 'react';

export type AuthContextValue = {
  session: AuthSessionResponse;
  refresh: () => Promise<void>;
  signOut: () => Promise<void>;
};

export const AuthContext = createContext<AuthContextValue | null>(null);

export function useAuth(): AuthContextValue {
  const value = useContext(AuthContext);
  if (!value) {
    throw new Error('useAuth requires an authenticated session');
  }
  return value;
}
