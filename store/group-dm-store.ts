import { defineStore } from 'pinia';
import type { IGroupDM, IGroupDMMessage } from '~/types/group-dm';

interface State {
  groups: IGroupDM[];
  messages: Record<string, IGroupDMMessage[]>;
  unread: Record<string, number>;
  lastMessage: Record<string, IGroupDMMessage | null>;
  loading: boolean;
  error: string | null;
}

export const useGroupDMStore = defineStore('group-dm', {
  state: (): State => ({
    groups: [],
    messages: {},
    unread: {},
    lastMessage: {},
    loading: false,
    error: null,
  }),
  actions: {
    async fetchGroups() {
      this.loading = true;
      try {
        const data = await $fetch('/api/group-dms');
        this.groups = data?.groups || [];
        // Optionally fetch last message for each group
        for (const group of this.groups) {
          if (group._id) {
            await this.fetchLastMessage(group._id);
          }
        }
        this.error = null;
      } catch (e: any) {
        this.error = e.message || 'Failed to fetch group DMs.';
      } finally {
        this.loading = false;
      }
    },
    async fetchMessages(groupId: string, opts: { limit?: number; after?: string } = {}) {
      this.loading = true;
      try {
        const data = await $fetch(`/api/group-dms/${groupId}/messages`, { params: opts });
        this.messages[groupId] = data?.messages || [];
        if (this.messages[groupId].length) {
          this.lastMessage[groupId] = this.messages[groupId][this.messages[groupId].length - 1];
        }
        this.unread[groupId] = 0;
        this.error = null;
      } catch (e: any) {
        this.error = e.message || 'Failed to fetch messages.';
      } finally {
        this.loading = false;
      }
    },
    async fetchLastMessage(groupId: string) {
      try {
        const data = await $fetch(`/api/group-dms/${groupId}/messages`, { params: { limit: 1 } });
        this.lastMessage[groupId] = data?.messages?.[0] || null;
      } catch {}
    },
    async sendMessage(groupId: string, content: string, attachments?: string[]) {
      try {
        const { data } = await useFetch(`/api/group-dms/${groupId}/message`, {
          method: 'POST',
          body: { content, attachments },
        });
        if (data.value?.message) {
          this.messages[groupId] = [...(this.messages[groupId] || []), data.value.message];
          this.lastMessage[groupId] = data.value.message;
        }
        this.error = null;
      } catch (e: any) {
        this.error = e.message || 'Failed to send message.';
      }
    },
    async createGroup(payload: { name?: string; memberIds: string[]; avatarUrl?: string }) {
      try {
        const { data } = await useFetch('/api/group-dms', {
          method: 'POST',
          body: payload,
        });
        if (data.value?.group) {
          this.groups.push(data.value.group);
        }
        this.error = null;
      } catch (e: any) {
        this.error = e.message || 'Failed to create group DM.';
      }
    },
    async updateGroup(groupId: string, payload: { name?: string; memberIds?: string[]; avatarUrl?: string }) {
      try {
        const { data } = await useFetch(`/api/group-dms/${groupId}`, {
          method: 'PUT',
          body: payload,
        });
        if (data.value?.group) {
          const idx = this.groups.findIndex(g => g._id === groupId);
          if (idx !== -1) this.groups[idx] = data.value.group;
        }
        this.error = null;
      } catch (e: any) {
        this.error = e.message || 'Failed to update group DM.';
      }
    },
    async leaveOrDeleteGroup(groupId: string) {
      try {
        await useFetch(`/api/group-dms/${groupId}`, { method: 'DELETE' });
        this.groups = this.groups.filter(g => g._id !== groupId);
        delete this.messages[groupId];
        delete this.unread[groupId];
        delete this.lastMessage[groupId];
        this.error = null;
      } catch (e: any) {
        this.error = e.message || 'Failed to leave/delete group DM.';
      }
    },
    incrementUnread(groupId: string) {
      this.unread[groupId] = (this.unread[groupId] || 0) + 1;
    },
    markAsRead(groupId: string) {
      this.unread[groupId] = 0;
    },
  },
}); 