<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import FriendCard from './friend-card.vue'
import { useSocket } from '../composables/useSocket'
import { useUserStore } from '~/store/user-store'

const userStore = useUserStore()
const isLoading = ref(true)
const hasError = ref(false)
const errorMessage = ref('')
const actionLoading = ref<string | null>(null)
const toast = useToast()
const {
  onFriendRemoved,
  onUserBlocked,
  onFriendRequestSent,
  onFriendRequestAccepted,
  onFriendRequestRejected,
  onUserUnblocked
} = useSocket(userStore.token)

interface FriendCardUser {
  id: string
  username: string
  tag: string
  avatarUrl: string
  bio?: string
  status?: 'online' | 'idle' | 'dnd' | 'offline'
  statusEmoji?: string
  statusText?: string
}

interface MutualServer {
  id: string
  name: string
  avatarUrl: string
}

const friends = ref<FriendCardUser[]>([])
const mutualServersMap = ref<Record<string, MutualServer[]>>({})

async function fetchFriends() {
  isLoading.value = true
  hasError.value = false
  try {
    const data = await $fetch('/api/friends')
    if (data && data.friends) {
      friends.value = data.friends
      // Fetch mutual servers for each friend
      for (const friend of friends.value) {
        const mutualData = await $fetch(`/api/friends/${friend.id}/mutual-servers`)
        mutualServersMap.value[friend.id] = mutualData?.mutualServers || []
      }
    }
  } catch (err: any) {
    hasError.value = true
    errorMessage.value = err.message || 'Failed to load friends.'
  } finally {
    isLoading.value = false
  }
}

function handleMessage(user: FriendCardUser) {
  // Navigate to DM or open chat
  // Example: useRouter().push(`/dms/${user.id}`)
  toast.add({
    title: `Opening chat with ${user.username}`,
    description: 'This will open a new chat window with the user.',
    color: 'primary',
    icon: 'i-heroicons-chat-bubble-left-right'
  })
}

async function handleRemove(user: FriendCardUser) {
  if (!confirm(`Remove ${user.username} from your friends?`)) return
  actionLoading.value = user.id
  try {
    await $fetch(`/api/friends/${user.id}/remove`, { method: 'POST' })
    toast.add({
      title: `Removed ${user.username} from friends.`,
      color: 'success',
      icon: 'i-heroicons-check-circle'
    })
    await fetchFriends()
  } catch (err: any) {
    toast.add({
      title: err.data?.error || 'Failed to remove friend.',
      color: 'error',
      icon: 'i-heroicons-x-circle'
    })
  } finally {
    actionLoading.value = null
  }
}

async function handleBlock(user: FriendCardUser) {
  if (!confirm(`Block ${user.username}? They will not be able to message you.`)) return
  actionLoading.value = user.id
  try {
    await $fetch(`/api/friends/${user.id}/block`, { method: 'POST' })
    toast.add({
      title: `Blocked ${user.username}.`,
      color: 'success',
      icon: 'i-heroicons-check-circle'
    })
    await fetchFriends()
  } catch (err: any) {
    toast.add({
      title: err.data?.error || 'Failed to block user.',
      color: 'error',
      icon: 'i-heroicons-x-circle'
    })
  } finally {
    actionLoading.value = null
  }
}

function handleFriendRemoved({ friendId }: { friendId: string }) {
  toast.add({
    title: 'A friend was removed.',
    color: 'info',
    icon: 'i-heroicons-information-circle'
  })
  fetchFriends()
}
function handleUserBlocked({ blockerId }: { blockerId: string }) {
  toast.add({
    title: 'You were blocked by a user.',
    color: 'info',
    icon: 'i-heroicons-information-circle'
  })
  fetchFriends()
}
function handleFriendRequestSent({ from, username }: { from: string, username: string }) {
  toast.add({
    title: `${username} sent you a friend request!`,
    color: 'info',
    icon: 'i-heroicons-information-circle'
  })
}
function handleFriendRequestAccepted({ from, username }: { from: string, username?: string }) {
  toast.add({
    title: `${username || 'A user'} accepted your friend request!`,
    color: 'success',
    icon: 'i-heroicons-check-circle'
  })
  fetchFriends()
}
function handleFriendRequestRejected({ from, username }: { from: string, username?: string }) {
  toast.add({
    title: `${username || 'A user'} rejected your friend request.`,
    color: 'error',
    icon: 'i-heroicons-x-circle'
  })
}
function handleUserUnblocked({ by }: { by: string }) {
  toast.add({
    title: 'You were unblocked!',
    color: 'info',
    icon: 'i-heroicons-information-circle'
  })
  fetchFriends()
}

onMounted(() => {
  fetchFriends()
  onFriendRemoved(handleFriendRemoved)
  onUserBlocked(handleUserBlocked)
  onFriendRequestSent(handleFriendRequestSent)
  onFriendRequestAccepted(handleFriendRequestAccepted)
  onFriendRequestRejected(handleFriendRequestRejected)
  onUserUnblocked(handleUserUnblocked)
})
onUnmounted(() => {
  // Optionally: remove listeners if needed
})
</script>

<template>
  <div>
    <div v-if="isLoading" class="text-center py-8 text-gray-400">Loading friends...</div>
    <div v-else-if="hasError" class="text-center py-8 text-red-500">{{ errorMessage }}</div>
    <div v-else class="flex flex-col gap-4">
      <FriendCard
        v-for="friend in friends"
        :key="friend.id"
        :user="friend"
        :mutual-servers="mutualServersMap[friend.id]"
        @message="handleMessage"
        @remove="() => handleRemove(friend)"
        @block="() => handleBlock(friend)"
      >
        <template #actions>
          <span v-if="actionLoading === friend.id" class="ml-2 text-xs text-gray-400">Processing...</span>
        </template>
      </FriendCard>
      <div v-if="!friends.length" class="text-center text-gray-400 py-8">No friends found.</div>
    </div>
  </div>
</template> 