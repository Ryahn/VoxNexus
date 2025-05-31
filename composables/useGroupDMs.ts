import { storeToRefs } from 'pinia';
import { useGroupDMStore } from '~/store/group-dm-store';
import type { IGroupDM } from '~/types/group-dm';

export function useGroupDMs() {
  const store = useGroupDMStore();
  const { groups, loading, error } = storeToRefs(store);

  async function fetchGroups() {
    await store.fetchGroups();
  }
  async function createGroup(payload: { name?: string; memberIds: string[]; avatarUrl?: string }) {
    await store.createGroup(payload);
  }
  async function updateGroup(groupId: string, payload: { name?: string; memberIds?: string[]; avatarUrl?: string }) {
    await store.updateGroup(groupId, payload);
  }
  async function leaveOrDeleteGroup(groupId: string) {
    await store.leaveOrDeleteGroup(groupId);
  }

  return {
    groups,
    loading,
    error,
    fetchGroups,
    createGroup,
    updateGroup,
    leaveOrDeleteGroup,
  };
} 