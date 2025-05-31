<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import FriendCard from './friend-card.vue'
import { useToast } from '../composables/useToast'
import { useSocket } from '../composables/useSocket'
import { useUserStore } from '~/store/user-store'

const userStore = useUserStore()
const token = userStore.token

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
const isLoading = ref(true)
const hasError = ref(false)
const errorMessage = ref('')
const actionLoading = ref<string | null>(null)
const { showToast } = useToast()
const {
  onFriendRemoved,
  onUserBlocked,
  onFriendRequestSent,
  onFriendRequestAccepted,
  onFriendRequestRejected,
  onUserUnblocked
} = useSocket(token)

async function fetchFriends() {
  isLoading.value = true
  hasError.value = false
  try {
    const { data } = await useFetch('/api/friends')
    if (data.value && data.value.friends) {
      friends.value = data.value.friends
      // Fetch mutual servers for each friend
      for (const friend of friends.value) {
        const { data: mutualData } = await useFetch(`/api/friends/${friend.id}/mutual-servers`)
        mutualServersMap.value[friend.id] = mutualData.value?.mutualServers || []
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
  showToast(`Opening chat with ${user.username}`, 'info')
}

async function handleRemove(user: FriendCardUser) {
  if (!confirm(`Remove ${user.username} from your friends?`)) return
  actionLoading.value = user.id
  try {
    await $fetch(`/api/friends/${user.id}/remove`, { method: 'POST' })
    showToast(`Removed ${user.username} from friends.`, 'success')
    await fetchFriends()
  } catch (err: any) {
    showToast(err.data?.error || 'Failed to remove friend.', 'error')
  } finally {
    actionLoading.value = null
  }
}

async function handleBlock(user: FriendCardUser) {
  if (!confirm(`Block ${user.username}? They will not be able to message you.`)) return
  actionLoading.value = user.id
  try {
    await $fetch(`/api/friends/${user.id}/block`, { method: 'POST' })
    showToast(`Blocked ${user.username}.`, 'success')
    await fetchFriends()
  } catch (err: any) {
    showToast(err.data?.error || 'Failed to block user.', 'error')
  } finally {
    actionLoading.value = null
  }
}

function handleFriendRemoved({ friendId }: { friendId: string }) {
  showToast('A friend was removed.', 'info')
  fetchFriends()
}
function handleUserBlocked({ blockerId }: { blockerId: string }) {
  showToast('You were blocked by a user.', 'info')
  fetchFriends()
}
function handleFriendRequestSent({ from, username }: { from: string, username: string }) {
  showToast(`${username} sent you a friend request!`, 'info')
}
function handleFriendRequestAccepted({ from, username }: { from: string, username?: string }) {
  showToast(`${username || 'A user'} accepted your friend request!`, 'success')
  fetchFriends()
}
function handleFriendRequestRejected({ from, username }: { from: string, username?: string }) {
  showToast(`${username || 'A user'} rejected your friend request.`, 'error')
}
function handleUserUnblocked({ by }: { by: string }) {
  showToast('You were unblocked!', 'info')
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