import { storeToRefs } from 'pinia';
import { useGroupDMStore } from '~/store/group-dm-store';
import type { IGroupDMMessage } from '~/types/group-dm';

export function useGroupDMChat(groupId: string) {
  const store = useGroupDMStore();
  const { messages, loading, error } = storeToRefs(store);

  async function fetchMessages(opts: { limit?: number; after?: string } = {}) {
    await store.fetchMessages(groupId, opts);
  }
  async function sendMessage(content: string, attachments?: string[]) {
    await store.sendMessage(groupId, content, attachments);
  }

  return {
    messages: computed(() => messages.value[groupId] || []),
    loading,
    error,
    fetchMessages,
    sendMessage,
  };
} 