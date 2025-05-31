<script setup lang="ts">
import { onMounted, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useGroupDMs } from '~/composables/useGroupDMs';
import { useGroupDMStore } from '~/store/group-dm-store';
import { useUserStore } from '~/store/user-store';

const router = useRouter();
const { groups, fetchGroups, loading } = useGroupDMs();
const groupDMStore = useGroupDMStore();
const userStore = useUserStore();
const unread = computed(() => groupDMStore.unread);
const lastMessage = computed(() => groupDMStore.lastMessage);

onMounted(() => {
  fetchGroups();
  userStore.fetchAllUsers();
});

function handleSelect(groupId: string) {
  router.push(`/group-dms/${groupId}`);
  groupDMStore.markAsRead(groupId);
}

function getUser(id: string) {
  return userStore.allUsers.find(u => u.id === id);
}
</script>

<template>
  <aside class="w-64 bg-gray-800 h-full flex flex-col border-r border-gray-700">
    <div class="p-4 text-lg font-bold text-white border-b border-gray-700">Group DMs</div>
    <div class="flex-1 overflow-y-auto">
      <div v-if="loading" class="p-4 text-gray-400">Loading...</div>
      <div v-else-if="!groups.length" class="p-4 text-gray-400">No group DMs</div>
      <ul>
        <li
          v-for="group in groups"
          :key="group._id"
          class="flex items-center px-4 py-2 cursor-pointer hover:bg-gray-700 transition"
          @click="handleSelect(group._id)"
        >
          <div class="flex items-center mr-3">
            <template v-if="group.avatarUrl">
              <img :src="group.avatarUrl" class="w-10 h-10 rounded-full object-cover" />
            </template>
            <template v-else>
              <template v-for="(id, idx) in group.memberIds.slice(0, 3)" :key="id">
                <img v-if="getUser(id)?.avatarUrl" :src="getUser(id).avatarUrl" class="w-7 h-7 rounded-full object-cover -ml-2 border-2 border-gray-800" />
                <div v-else class="w-7 h-7 rounded-full bg-gray-600 flex items-center justify-center text-white text-xs font-bold -ml-2 border-2 border-gray-800">
                  {{ getUser(id)?.username?.[0] || '?' }}
                </div>
              </template>
              <span v-if="group.memberIds.length > 3" class="ml-1 text-xs text-gray-400">+{{ group.memberIds.length - 3 }}</span>
            </template>
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-white font-semibold truncate">
              {{ group.name || group.memberIds.map(id => getUser(id)?.username || id).join(', ') }}
            </div>
            <div class="text-xs text-gray-400 truncate">{{ group.memberIds.length }} members</div>
            <div v-if="lastMessage[group._id]" class="text-xs text-gray-300 truncate mt-0.5">
              <span class="font-medium text-gray-400">{{ lastMessage[group._id]?.content }}</span>
              <span class="ml-2 text-gray-500">{{ lastMessage[group._id]?.createdAt ? new Date(lastMessage[group._id].createdAt).toLocaleTimeString() : '' }}</span>
            </div>
          </div>
          <span v-if="unread[group._id] > 0" class="ml-2 bg-blue-600 text-white text-xs rounded-full px-2 py-0.5">{{ unread[group._id] }}</span>
        </li>
      </ul>
    </div>
    <slot name="footer"></slot>
  </aside>
</template>

<style scoped>
.w-64 {
  width: 16rem;
}
</style> 