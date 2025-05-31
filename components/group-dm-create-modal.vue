<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useGroupDMs } from '~/composables/useGroupDMs';
import { useUserStore } from '~/store/user-store';

const emit = defineEmits(['close', 'created']);
const { createGroup, loading, error } = useGroupDMs();
const userStore = useUserStore();
const allUsers = computed(() => userStore.allUsers?.filter(u => u.id !== userStore.user?.id) || []);

const name = ref('');
const avatarUrl = ref('');
const selected = ref<string[]>([]);

onMounted(() => {
  userStore.fetchAllUsers();
});

async function handleCreate() {
  if (selected.value.length < 1) return;
  const memberIds = [userStore.user.id, ...selected.value];
  await createGroup({ name: name.value, memberIds, avatarUrl: avatarUrl.value });
  emit('created');
  emit('close');
}
</script>

<template>
  <div class="fixed inset-0 bg-black bg-opacity-60 flex items-center justify-center z-50">
    <div class="bg-gray-800 rounded-lg shadow-lg w-full max-w-md p-6">
      <div class="flex justify-between items-center mb-4">
        <h2 class="text-xl font-bold text-white">Create Group DM</h2>
        <button @click="$emit('close')" class="text-gray-400 hover:text-white">&times;</button>
      </div>
      <div class="mb-4">
        <label class="block text-gray-300 mb-1">Group Name</label>
        <input v-model="name" class="w-full rounded bg-gray-700 text-white px-3 py-2 focus:outline-none" maxlength="32" />
      </div>
      <div class="mb-4">
        <label class="block text-gray-300 mb-1">Avatar URL (optional)</label>
        <input v-model="avatarUrl" class="w-full rounded bg-gray-700 text-white px-3 py-2 focus:outline-none" />
      </div>
      <div class="mb-4">
        <label class="block text-gray-300 mb-1">Add Members (up to 19)</label>
        <select v-model="selected" multiple class="w-full rounded bg-gray-700 text-white px-3 py-2 focus:outline-none h-32">
          <option v-for="user in allUsers" :key="user.id" :value="user.id">{{ user.username }}</option>
        </select>
      </div>
      <div class="flex justify-end space-x-2">
        <button @click="$emit('close')" class="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-700">Cancel</button>
        <button @click="handleCreate" :disabled="loading || selected.length < 1 || selected.length > 19" class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50">Create</button>
      </div>
      <div v-if="error" class="text-red-400 mt-2">{{ error }}</div>
    </div>
  </div>
</template> 