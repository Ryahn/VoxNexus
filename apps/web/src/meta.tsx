import { getMeta, type MetaResponse } from '@voxnexus/api-client';
import { createContext, type ReactNode, useContext, useEffect, useState } from 'react';

const MetaContext = createContext<MetaResponse | null>(null);

export function MetaProvider({ children }: { children: ReactNode }) {
  const [meta, setMeta] = useState<MetaResponse | null>(null);

  useEffect(() => {
    let cancelled = false;
    getMeta()
      .then((result) => {
        if (!cancelled) {
          setMeta(result.data ?? null);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setMeta(null);
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  return <MetaContext.Provider value={meta}>{children}</MetaContext.Provider>;
}

export function useMeta(): MetaResponse | null {
  return useContext(MetaContext);
}
