<template>
  <div class="flex flex-col items-center justify-center h-full text-center">
    <h1 class="text-2xl mb-2">Welcome to Discord Clone</h1>
    <div v-if="servers.length">
      <h2 class="text-lg mb-2">Your Servers:</h2>
      <ul>
        <li v-for="server in servers" :key="server.id" class="mb-2">
          <strong>{{ server.name }}</strong>
          <ul>
            <li v-for="channel in server.channels" :key="channel.id">
              # {{ channel.name }}
            </li>
          </ul>
        </li>
      </ul>
    </div>
    <div v-else>
      <p class="text-[#b9bbbe] mb-4">You are not a member of any servers yet.</p>
      <div class="flex gap-4">
        <button @click="showCreate = true" class="bg-indigo-600 hover:bg-indigo-700 text-white px-4 py-2 rounded">Create Server</button>
        <button @click="showJoin = true" class="bg-gray-700 hover:bg-gray-600 text-white px-4 py-2 rounded">Join Server</button>
      </div>
    </div>

    <!-- Create Server Modal -->
    <div v-if="showCreate" class="fixed inset-0 flex items-center justify-center bg-black bg-opacity-50 z-50">
      <div class="bg-gray-800 p-6 rounded shadow-lg w-80">
        <h2 class="text-lg mb-4 text-white">Create a Server</h2>
        <input v-model="newServerName" placeholder="Server Name" class="w-full mb-4 px-3 py-2 rounded bg-gray-700 text-white" />
        <input v-model="newServerDescription" placeholder="Server Description" class="w-full mb-4 px-3 py-2 rounded bg-gray-700 text-white" />
        <select v-model="newServerType" class="w-full mb-4 px-3 py-2 rounded bg-gray-700 text-white">
          <option value="public">Public</option>
          <option value="private">Private</option>
          <option value="community">Community</option>
        </select>
        <div class="flex flex-col gap-2">
          <input type="file" @change="handleIconUpload" class="w-full mb-4 px-3 py-2 rounded bg-gray-700 text-white" />
          <input type="file" @change="handleBannerUpload" class="w-full mb-4 px-3 py-2 rounded bg-gray-700 text-white" />
        </div>
        <div class="flex flex-col gap-2">
          <input type="checkbox" v-model="newServerIsPublic" class="w-full mb-4 px-3 py-2 rounded bg-gray-700 text-white" />
          <input type="checkbox" v-model="newServerIsNsfw" class="w-full mb-4 px-3 py-2 rounded bg-gray-700 text-white" />
        </div>
        <div class="flex justify-end gap-2">
          <button @click="showCreate = false" class="px-3 py-1 bg-gray-600 text-white rounded">Cancel</button>
          <button @click="createServer" class="px-3 py-1 bg-indigo-600 text-white rounded">Create</button>
        </div>
      </div>
    </div>

    <!-- Join Server Modal -->
    <div v-if="showJoin" class="fixed inset-0 flex items-center justify-center bg-black bg-opacity-50 z-50">
      <div class="bg-gray-800 p-6 rounded shadow-lg w-80">
        <h2 class="text-lg mb-4 text-white">Join a Server</h2>
        <input v-model="joinCode" placeholder="Invite Code" class="w-full mb-4 px-3 py-2 rounded bg-gray-700 text-white" />
        <div class="flex justify-end gap-2">
          <button @click="showJoin = false" class="px-3 py-1 bg-gray-600 text-white rounded">Cancel</button>
          <button @click="joinServer" class="px-3 py-1 bg-indigo-600 text-white rounded">Join</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useServerStore } from '@/stores/server'

const serverStore = useServerStore()
const servers = serverStore.servers

const showCreate = ref(false)
const showJoin = ref(false)
const newServerName = ref('')
const newServerDescription = ref('')
const newServerType = ref<'public' | 'private' | 'community'>('public')
const newServerIsPublic = ref(false)
const newServerIsNsfw = ref(false)
const newServerIcon = ref<File | null>(null)
const newServerBanner = ref<File | null>(null)

const handleIconUpload = (e: Event) => {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (file) {
    newServerIcon.value = file
  }
}

const handleBannerUpload = (e: Event) => {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (file) {
    newServerBanner.value = file
  }
}

const joinCode = ref('')

onMounted(() => {
  serverStore.fetchServers()
})

const createServer = async () => {
  if (!newServerName.value.trim()) return
  await serverStore.createServer({
    name: newServerName.value,
    icon: newServerIcon.value || undefined,
    banner: newServerBanner.value || undefined,
    description: newServerDescription.value,
    isPublic: newServerIsPublic.value,
    isNsfw: newServerIsNsfw.value,
    type: newServerType.value
  })
  showCreate.value = false
  newServerName.value = ''
}

const joinServer = async () => {
  // Implement join logic here (e.g., call an API with joinCode)
  // For now, just close the modal
  showJoin.value = false
  joinCode.value = ''
}
</script> 