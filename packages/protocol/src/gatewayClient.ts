import type { Envelope, HelloPayload } from './generated/gateway';

export type {
  DevPingPayload,
  DevPongPayload,
  Envelope,
  EventType,
  HelloPayload,
} from './generated/gateway';

export const GATEWAY_SUBPROTOCOL = 'voxnexus.gateway.v1';

export type GatewayClient = {
  connect: () => void;
  disconnect: () => void;
  readonly readyState: number;
};

export type GatewayClientOptions = {
  url: string;
  onHello?: (hello: HelloPayload, envelope: Envelope) => void;
  onEnvelope?: (envelope: Envelope) => void;
  onClose?: () => void;
  onError?: (error: Event) => void;
};

/**
 * Minimal WebSocket gateway stub: connect, receive HELLO, disconnect.
 * Chat traffic is not on this socket until F035.
 */
export function createGatewayClient(options: GatewayClientOptions): GatewayClient {
  let socket: WebSocket | null = null;

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
  };
}
