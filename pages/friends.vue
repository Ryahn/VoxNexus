<template>
  <div class="max-w-2xl mx-auto py-8">
    <h1 class="text-2xl font-bold mb-4 text-white">Friends</h1>
    <div class="mb-6">
      <input v-model="searchQuery" @keyup.enter="handleSearch" class="w-full p-2 rounded bg-gray-700 text-white" placeholder="Search users..." />
      <div v-if="searchResults.length" class="mt-2 bg-gray-800 rounded p-2">
        <div v-for="user in searchResults" :key="user._id" class="flex items-center justify-between py-1">
          <div class="flex items-center">
            <img :src="user.avatarUrl || 'https://ui-avatars.com/api/?name=' + user.username" class="w-8 h-8 rounded-full mr-2" />
            <span class="text-white">{{ user.username }}</span>
          </div>
          <button @click="sendRequest(user._id)" class="px-2 py-1 bg-green-600 text-white rounded hover:bg-green-700">Add Friend</button>
        </div>
      </div>
    </div>
    <div class="mb-8">
      <h2 class="text-lg font-semibold text-white mb-2">Incoming Requests</h2>
      <div v-if="friendStore.incoming.length" class="space-y-2">
        <div v-for="req in friendStore.incoming" :key="req._id" class="flex items-center justify-between bg-gray-800 p-2 rounded">
          <div class="flex items-center">
            <img :src="req.from?.avatarUrl || 'https://ui-avatars.com/api/?name=' + req.from?.username" class="w-8 h-8 rounded-full mr-2" />
            <span class="text-white">{{ req.from?.username }}</span>
          </div>
          <div>
            <button @click="acceptRequest(req._id)" class="px-2 py-1 bg-green-600 text-white rounded mr-2">Accept</button>
            <button @click="rejectRequest(req._id)" class="px-2 py-1 bg-red-600 text-white rounded">Reject</button>
          </div>
        </div>
      </div>
      <div v-else class="text-gray-400">No incoming requests.</div>
    </div>
    <div class="mb-8">
      <h2 class="text-lg font-semibold text-white mb-2">Outgoing Requests</h2>
      <div v-if="friendStore.outgoing.length" class="space-y-2">
        <div v-for="req in friendStore.outgoing" :key="req._id" class="flex items-center bg-gray-800 p-2 rounded">
          <img :src="req.to?.avatarUrl || 'https://ui-avatars.com/api/?name=' + req.to?.username" class="w-8 h-8 rounded-full mr-2" />
          <span class="text-white">{{ req.to?.username }}</span>
          <span class="ml-auto text-xs text-gray-400">Pending</span>
        </div>
      </div>
      <div v-else class="text-gray-400">No outgoing requests.</div>
    </div>
    <div>
      <h2 class="text-lg font-semibold text-white mb-2">Your Friends</h2>
      <div v-if="friendStore.friends.length" class="space-y-2">
        <div v-for="friend in friendStore.friends" :key="friend._id" class="flex items-center bg-gray-800 p-2 rounded">
          <img :src="friend.avatarUrl || 'https://ui-avatars.com/api/?name=' + friend.username" class="w-8 h-8 rounded-full mr-2" />
          <span class="text-white">{{ friend.username }}</span>
          <span class="ml-2 text-xs text-gray-400">{{ friend.status }}</span>
          <NuxtLink :to="`/dms/${friend._id}`" class="ml-auto px-2 py-1 bg-blue-600 text-white rounded hover:bg-blue-700">Message</NuxtLink>
        </div>
      </div>
      <div v-else class="text-gray-400">No friends yet.</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useFriendStore } from '~/store/friend-store'
import { useUserStore } from '~/store/user-store'

const friendStore = useFriendStore()
const userStore = useUserStore()
const searchQuery = ref('')
const searchResults = ref<any[]>([])

onMounted(() => {
  friendStore.fetchFriends(userStore.token)
})

async function handleSearch() {
  if (!searchQuery.value.trim()) return
  const res = await $fetch('/api/users/search', {
    query: { q: searchQuery.value },
    headers: { Authorization: `Bearer ${userStore.token}` },
  })
  searchResults.value = (res as any).users || []
}

async function sendRequest(userId: string) {
  await friendStore.sendRequest(userId, userStore.token)
  await friendStore.fetchFriends(userStore.token)
}
async function acceptRequest(requestId: string) {
  await friendStore.acceptRequest(requestId, userStore.token)
  await friendStore.fetchFriends(userStore.token)
}
async function rejectRequest(requestId: string) {
  await friendStore.rejectRequest(requestId, userStore.token)
  await friendStore.fetchFriends(userStore.token)
}
</script> 