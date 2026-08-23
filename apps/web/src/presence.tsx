import { listPresence, updateMyProfile } from '@voxnexus/api-client';
import type { PresenceUpdatePayload } from '@voxnexus/protocol';
import { createGatewayClient } from '@voxnexus/protocol';
import {
  createContext,
  type ReactNode,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from 'react';
import { useAuth } from './auth';
import { readApiErrorMessage } from './lib/apiError';

export type PresenceState = 'online' | 'idle' | 'dnd' | 'invisible' | 'offline';

export type PresenceEntry = {
  accountId: string;
  status: PresenceState;
  customStatus: string;
};

type PresenceContextValue = {
  self: PresenceEntry | null;
  online: PresenceEntry[];
  setStatus: (status: PresenceState, customStatus?: string) => Promise<void>;
};

const credentials = { credentials: 'include' as const };

const PresenceContext = createContext<PresenceContextValue | null>(null);

function mapStatus(status: string): PresenceState {
  if (
    status === 'online' ||
    status === 'idle' ||
    status === 'dnd' ||
    status === 'invisible' ||
    status === 'offline'
  ) {
    return status;
  }
  return 'offline';
}

function entryFromPayload(payload: PresenceUpdatePayload): PresenceEntry {
  return {
    accountId: payload.account_id,
    status: mapStatus(payload.status),
    customStatus: payload.custom_status,
  };
}

function upsert(entries: PresenceEntry[], update: PresenceEntry): PresenceEntry[] {
  const next = entries.filter((entry) => entry.accountId !== update.accountId);
  if (update.status !== 'offline') {
    next.push(update);
  }
  return next;
}

export function PresenceProvider({ children }: { children: ReactNode }) {
  const { session } = useAuth();
  const accountId = session.account.id;
  const [self, setSelf] = useState<PresenceEntry | null>(null);
  const [online, setOnline] = useState<PresenceEntry[]>([]);

  const applyUpdate = useCallback(
    (payload: PresenceUpdatePayload) => {
      const entry = entryFromPayload(payload);
      if (payload.account_id === accountId) {
        setSelf(entry);
      }
      if (entry.status === 'offline' || entry.status === 'invisible') {
        setOnline((prev) => prev.filter((row) => row.accountId !== entry.accountId));
        return;
      }
      setOnline((prev) => upsert(prev, entry));
    },
    [accountId],
  );

  useEffect(() => {
    let cancelled = false;
    listPresence(credentials)
      .then((result) => {
        if (cancelled || !result.data) {
          return;
        }
        const entries = result.data.presences.map((row) => ({
          accountId: row.account_id,
          status: mapStatus(row.status),
          customStatus: row.custom_status,
        }));
        setOnline(entries.filter((row) => row.status !== 'offline' && row.status !== 'invisible'));
      })
      .catch(() => undefined);
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    const origin = window.location.origin;
    const client = createGatewayClient({
      url: `${origin}/api/v1/gateway`,
      onEnvelope: (envelope) => {
        if (envelope.event_type === 'PRESENCE_SYNC') {
          const presences =
            (envelope.payload as { presences?: PresenceUpdatePayload[] }).presences ?? [];
          const mapped = presences.map(entryFromPayload);
          setSelf(mapped.find((row) => row.accountId === accountId) ?? null);
          setOnline(
            mapped.filter(
              (row) =>
                row.accountId !== accountId &&
                row.status !== 'offline' &&
                row.status !== 'invisible',
            ),
          );
        }
        if (envelope.event_type === 'PRESENCE_UPDATE') {
          applyUpdate(envelope.payload as PresenceUpdatePayload);
        }
      },
    });
    client.connect();
    return () => client.disconnect();
  }, [accountId, applyUpdate]);

  const setStatus = useCallback(
    async (status: PresenceState, customStatus?: string) => {
      const body: {
        presence_status?: 'online' | 'idle' | 'dnd' | 'invisible';
        custom_status?: string;
      } = {};
      if (status !== 'offline') {
        body.presence_status = status;
      }
      if (customStatus !== undefined) {
        body.custom_status = customStatus;
      }
      const result = await updateMyProfile({ body, ...credentials });
      if (result.error || !result.data) {
        throw new Error(readApiErrorMessage(result.error, 'Could not update presence.'));
      }
      setSelf({
        accountId,
        status: mapStatus(result.data.presence_status),
        customStatus: result.data.custom_status,
      });
    },
    [accountId],
  );

  const value = useMemo(
    () => ({
      self,
      online,
      setStatus,
    }),
    [self, online, setStatus],
  );

  return <PresenceContext.Provider value={value}>{children}</PresenceContext.Provider>;
}

export function usePresence(): PresenceContextValue {
  const value = useContext(PresenceContext);
  if (!value) {
    throw new Error('usePresence requires PresenceProvider');
  }
  return value;
}

export function presenceLabel(status: PresenceState, customStatus?: string): string {
  if (customStatus?.trim()) {
    return customStatus.trim();
  }
  switch (status) {
    case 'online':
      return 'Online';
    case 'idle':
      return 'Idle';
    case 'dnd':
      return 'Do not disturb';
    case 'invisible':
      return 'Invisible';
    default:
      return 'Offline';
  }
}
