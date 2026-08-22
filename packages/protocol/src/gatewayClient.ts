import type {
  Envelope,
  HelloPayload,
  ReadyPayload,
  ResumedPayload,
} from './generated/gateway';

export type {
  DevPingPayload,
  DevPongPayload,
  Envelope,
  EventType,
  HelloPayload,
  IdentifyPayload,
  InvalidSessionPayload,
  ReadyPayload,
  ResumePayload,
  ResumedPayload,
} from './generated/gateway';

export const GATEWAY_SUBPROTOCOL = 'voxnexus.gateway.v1';

export type GatewayClient = {
  connect: () => void;
  disconnect: () => void;
  identify: () => void;
  resume: (input: { session_id: string; last_sequence: number; resume_token: string }) => void;
  readonly readyState: number;
};

export type GatewayClientOptions = {
  url: string;
  onHello?: (hello: HelloPayload, envelope: Envelope) => void;
  onReady?: (ready: ReadyPayload, envelope: Envelope) => void;
  onResumed?: (resumed: ResumedPayload, envelope: Envelope) => void;
  onEnvelope?: (envelope: Envelope) => void;
  onClose?: () => void;
  onError?: (error: Event) => void;
  /** When true, send IDENTIFY automatically after HELLO. */
  autoIdentify?: boolean;
};

/**
 * Gateway client: connect (cookie-authenticated), HELLO → IDENTIFY → READY, optional RESUME.
 */
export function createGatewayClient(options: GatewayClientOptions): GatewayClient {
  let socket: WebSocket | null = null;
  const autoIdentify = options.autoIdentify !== false;

  function send(event_type: string, payload: unknown) {
    if (!socket || socket.readyState !== WebSocket.OPEN) {
      return;
    }
    const envelope = {
      event_id: crypto.randomUUID(),
      sequence: 0,
      event_type,
      timestamp: new Date().toISOString(),
      payload,
    };
    socket.send(JSON.stringify(envelope));
  }

  return {
    get readyState() {
      return socket?.readyState ?? WebSocket.CLOSED;
    },
    connect() {
      if (socket && socket.readyState < WebSocket.CLOSING) {
        return;
      }
      socket = new WebSocket(options.url, GATEWAY_SUBPROTOCOL);
      socket.addEventListener('message', (event) => {
        if (typeof event.data !== 'string') {
          return;
        }
        let envelope: Envelope;
        try {
          envelope = JSON.parse(event.data) as Envelope;
        } catch {
          return;
        }
        options.onEnvelope?.(envelope);
        if (envelope.event_type === 'HELLO') {
          options.onHello?.(envelope.payload as HelloPayload, envelope);
          if (autoIdentify) {
            send('IDENTIFY', {});
          }
        }
        if (envelope.event_type === 'READY') {
          options.onReady?.(envelope.payload as ReadyPayload, envelope);
        }
        if (envelope.event_type === 'RESUMED') {
          options.onResumed?.(envelope.payload as ResumedPayload, envelope);
        }
      });
      socket.addEventListener('close', () => {
        options.onClose?.();
      });
      socket.addEventListener('error', (error) => {
        options.onError?.(error);
      });
    },
    disconnect() {
      socket?.close();
      socket = null;
    },
    identify() {
      send('IDENTIFY', {});
    },
    resume(input) {
      send('RESUME', input);
    },
  };
}
