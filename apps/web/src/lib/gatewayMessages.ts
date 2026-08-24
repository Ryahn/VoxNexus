import type {
  Envelope,
  MessageCreatePayload,
  MessageDeletePayload,
  MessageUpdatePayload,
} from '@voxnexus/protocol';

type MessageCreateListener = (payload: MessageCreatePayload, envelope: Envelope) => void;
type MessageUpdateListener = (payload: MessageUpdatePayload, envelope: Envelope) => void;
type MessageDeleteListener = (payload: MessageDeletePayload, envelope: Envelope) => void;

const messageCreateListeners = new Set<MessageCreateListener>();
const messageUpdateListeners = new Set<MessageUpdateListener>();
const messageDeleteListeners = new Set<MessageDeleteListener>();

export function subscribeMessageCreate(listener: MessageCreateListener): () => void {
  messageCreateListeners.add(listener);
  return () => {
    messageCreateListeners.delete(listener);
  };
}

export function subscribeMessageUpdate(listener: MessageUpdateListener): () => void {
  messageUpdateListeners.add(listener);
  return () => {
    messageUpdateListeners.delete(listener);
  };
}

export function subscribeMessageDelete(listener: MessageDeleteListener): () => void {
  messageDeleteListeners.add(listener);
  return () => {
    messageDeleteListeners.delete(listener);
  };
}

export function dispatchGatewayEnvelope(envelope: Envelope): void {
  if (envelope.event_type === 'MESSAGE_CREATE') {
    const payload = envelope.payload as MessageCreatePayload;
    for (const listener of messageCreateListeners) {
      listener(payload, envelope);
    }
    return;
  }
  if (envelope.event_type === 'MESSAGE_UPDATE') {
    const payload = envelope.payload as MessageUpdatePayload;
    for (const listener of messageUpdateListeners) {
      listener(payload, envelope);
    }
    return;
  }
  if (envelope.event_type === 'MESSAGE_DELETE') {
    const payload = envelope.payload as MessageDeletePayload;
    for (const listener of messageDeleteListeners) {
      listener(payload, envelope);
    }
  }
}
