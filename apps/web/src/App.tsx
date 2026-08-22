import { getMeta, type MetaResponse } from '@voxnexus/api-client';
import { createGatewayClient } from '@voxnexus/protocol';
import { HelloPanel } from '@voxnexus/ui';
import { useEffect, useState } from 'react';

type CarrierState =
  | { status: 'loading' }
  | { status: 'live'; meta: MetaResponse }
  | { status: 'down'; detail: string };

function railMarkClass(status: CarrierState['status']): string {
  if (status === 'live') {
    return 'vn-rail-mark vn-rail-mark-live';
  }
  if (status === 'down') {
    return 'vn-rail-mark vn-rail-mark-down';
  }
  return 'vn-rail-mark';
}

function railLabel(status: CarrierState['status']): string {
  if (status === 'live') {
    return 'carrier live';
  }
  if (status === 'down') {
    return 'carrier down';
  }
  return 'carrier idle';
}

function instanceValue(carrier: CarrierState): string {
  if (carrier.status === 'live') {
    return carrier.meta.name;
  }
  if (carrier.status === 'down') {
    return 'unreachable';
  }
  return 'calling /api/v1/meta';
}

const gatewayDebug = import.meta.env.VITE_GATEWAY_DEBUG === 'true';

export function App() {
  const [carrier, setCarrier] = useState<CarrierState>({ status: 'loading' });
  const [gatewaySession, setGatewaySession] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    getMeta()
      .then((result) => {
        if (cancelled) {
          return;
        }
        if (result.data) {
          setCarrier({ status: 'live', meta: result.data });
          return;
        }
        setCarrier({
          status: 'down',
          detail: 'The API answered without instance identity.',
        });
      })
      .catch(() => {
        if (!cancelled) {
          setCarrier({
            status: 'down',
            detail: 'Start `cargo run -p voxnexus` on :8080, then reload.',
          });
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    if (!gatewayDebug) {
      return;
    }
    const protocol = window.location.protocol === 'https:' ? 'wss' : 'ws';
    const client = createGatewayClient({
      url: `${protocol}://${window.location.host}/api/v1/gateway`,
      onHello: (hello) => {
        setGatewaySession(hello.session_id);
      },
      onClose: () => {
        setGatewaySession(null);
      },
    });
    client.connect();
    return () => {
      client.disconnect();
    };
  }, []);

  return (
    <div className="vn-shell">
      <aside className="vn-rail" aria-label="Instance status">
        <span className={railMarkClass(carrier.status)} aria-hidden="true" />
        <span className="vn-rail-label">{railLabel(carrier.status)}</span>
      </aside>
      <main className="vn-main">
        <HelloPanel title="VoxNexus" kicker="Self-hostable community OS">
          <p>
            Discord-class chat and voice, Guilded-class Spaces, and a first-class app platform — on
            a server you run.
          </p>
          <dl className="vn-meta">
            <div>
              <dt>Instance</dt>
              <dd>{instanceValue(carrier)}</dd>
            </div>
            <div>
              <dt>Version</dt>
              <dd>{carrier.status === 'live' ? carrier.meta.version : '—'}</dd>
            </div>
            <div>
              <dt>License</dt>
              <dd>Source-available · not OSI</dd>
            </div>
          </dl>
          {carrier.status === 'down' ? <p className="vn-meta-note">{carrier.detail}</p> : null}
          {gatewayDebug ? (
            <p className="vn-meta-note">
              Gateway debug:{' '}
              {gatewaySession ? `HELLO session ${gatewaySession}` : 'connecting or down'}
            </p>
          ) : null}
        </HelloPanel>
      </main>
    </div>
  );
}
