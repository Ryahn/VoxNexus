<template>
  <div class="flex flex-col items-center min-h-screen bg-[var(--color-bg-main)] p-2 sm:p-8">
    <div class="w-full max-w-xl bg-[var(--color-bg-secondary)] rounded shadow-md flex flex-col h-[100dvh] sm:h-[80vh] border border-[var(--color-border)]">
      <div class="p-4 border-b border-[var(--color-border)] flex items-center">
        <img :src="otherUser.avatarUrl || 'https://ui-avatars.com/api/?name=' + otherUser.username" class="w-10 h-10 rounded-full mr-3 object-cover ring-2 relative" :class="getStatusBorderClass(otherUser.state?.statement)" alt="Avatar" />
        <span v-if="otherUser.state && otherUser.state.statement" class="absolute bottom-2 right-2 w-4 h-4 rounded-full border-2 border-[var(--color-bg-secondary)]" :class="getStatusDotClass(otherUser.state.statement)"></span>
        <div>
          <div class="text-[var(--color-text-main)] font-semibold text-base sm:text-lg">{{ otherUser.username }}</div>
          <div class="text-xs text-[var(--color-text-muted)]">{{ otherUser.status }}</div>
        </div>
      </div>
      <div class="flex-1 overflow-y-auto p-2 sm:p-4 space-y-4">
        <div v-for="(msg, i) in messages" :key="i" :class="msg.from === myId ? 'text-right' : 'text-left'">
          <div class="inline-block max-w-full sm:max-w-[70%] p-2 rounded-2xl shadow-md transition-all duration-150 hover:shadow-lg focus-within:ring-2 focus-within:ring-[var(--color-accent)]" :class="msg.from === myId ? 'bg-[var(--color-success)] text-white' : 'bg-[var(--color-bg-tertiary)] text-[var(--color-text-main)] border border-[var(--color-border)]'">
            <div class="text-xs text-[var(--color-text-secondary)] mb-1">{{ msg.from === myId ? 'You' : otherUser.username }}</div>
            <div class="break-words">{{ msg.content }}</div>
            <div class="flex items-center space-x-1 mt-1">
              <span
                v-for="reaction in msg.reactions || []"
                :key="reaction.emoji"
                class="inline-flex items-center px-2 py-0.5 rounded-full text-xs cursor-pointer relative"
                :class="[
                  'bg-gray-700',
                  reaction.userIds.includes(myId) ? 'ring-2 ring-yellow-400 bg-yellow-100 text-yellow-900' : '',
                  animatedReactions[`${i}:${reaction.emoji}`] ? 'scale-110 transition-transform duration-200' : ''
                ]"
                @click="handleReact(msg, reaction.emoji)"
                @mouseenter="(e) => showTooltip(e, reaction)"
                @mouseleave="hideTooltip"
              >
                {{ reaction.emoji }} {{ reaction.userIds.length }}
              </span>
              <!-- Tooltip -->
              <div
                v-if="tooltipReaction && tooltipUsers.length && msg.reactions && msg.reactions.some(r => r.emoji === tooltipReaction)"
                :style="{ position: 'fixed', left: tooltipX + 'px', top: (tooltipY + 20) + 'px', zIndex: 1000 }"
                class="px-3 py-2 rounded bg-gray-900 text-gray-100 text-xs shadow-lg border border-gray-700 pointer-events-none animate-fade-in"
              >
                <span v-for="(user, idx) in tooltipUsers" :key="user">
                  {{ user }}<span v-if="idx < tooltipUsers.length - 1">, </span>
                </span>
                <span v-if="tooltipUsers.length === 0">No reactions</span>
              </div>
            </div>
            <div class="text-xs text-[var(--color-text-muted)] mt-1">{{ msg.createdAt }}</div>
          </div>
        </div>
        <div v-if="isTyping" class="text-xs text-[var(--color-text-muted)] mt-2">{{ otherUser.username }} is typing...</div>
      </div>
      <form @submit.prevent="handleSend" class="flex items-center p-2 sm:p-4 border-t border-[var(--color-border)] sticky bottom-0 bg-[var(--color-bg-secondary)] z-20">
        <input v-model="input" @input="handleTyping" @blur="stopTyping" class="flex-1 p-3 rounded bg-[var(--color-bg-input)] text-[var(--color-text-main)] mr-2 text-base border border-[var(--color-border)]" placeholder="Type a message..." />
        <button type="submit" class="px-4 py-2 bg-[var(--color-success)] text-white rounded hover:bg-green-700 text-base">Send</button>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useUserStore } from '~/store/user-store'
import { useDm } from '~/composables/useDm'

const route = useRoute()
const userStore = useUserStore()
const otherUserId = computed(() => route.params.userId as string)
const myId = computed(() => userStore.user?.id || '')
const input = ref('')
const otherUser = ref({ username: 'User', avatarUrl: '', status: '' })

const { messages, typing: isTyping, fetchMessages, sendMessage, startTyping, stopTyping } = useDm(userStore.token, otherUserId.value, myId.value)

// Tooltip state
const tooltipReaction = ref<string | null>(null)
const tooltipX = ref(0)
const tooltipY = ref(0)
const tooltipUsers = ref<string[]>([])
function getUsernames(userIds: string[]): string[] {
  return userIds
    .map(id => userStore.allUsers.find(u => u.id === id)?.username || 'Unknown')
    .filter(Boolean)
}
function showTooltip(event: MouseEvent, reaction: any) {
  tooltipReaction.value = reaction.emoji
  tooltipUsers.value = getUsernames(reaction.userIds)
  tooltipX.value = event.clientX
  tooltipY.value = event.clientY
}
function hideTooltip() {
  tooltipReaction.value = null
  tooltipUsers.value = []
}
// Animation state for reactions
const animatedReactions = ref<{ [key: string]: boolean }>({})
watch(
  () => messages.value.map((m: any, i: number) => (m.reactions || []).map((r: any) => `${i}:${r.emoji}:${r.userIds.length}`).join(',')).join(','),
  () => {
    messages.value.forEach((msg: any, i: number) => {
      (msg.reactions || []).forEach((r: any) => {
        animatedReactions.value[`${i}:${r.emoji}`] = true
        setTimeout(() => (animatedReactions.value[`${i}:${r.emoji}`] = false), 300)
      })
    })
  }
)

onMounted(async () => {
  await fetchMessages()
  // Optionally fetch other user info from API
})

function handleSend() {
  if (!input.value.trim()) return
  sendMessage(input.value)
  input.value = ''
  stopTyping()
}

let typingTimeout: any = null
function handleTyping() {
  startTyping()
  clearTimeout(typingTimeout)
  typingTimeout = setTimeout(() => {
    stopTyping()
  }, 2000)
}

function getStatusBorderClass(state?: string) {
  switch (state) {
    case 'online': return 'ring-green-500';
    case 'idle': return 'ring-yellow-400';
    case 'dnd': return 'ring-red-500';
    default: return 'ring-gray-500';
  }
}
function getStatusDotClass(state?: string) {
  switch (state) {
    case 'online': return 'bg-green-500';
    case 'idle': return 'bg-yellow-400';
    case 'dnd': return 'bg-red-500';
    default: return 'bg-gray-500';
  }
}
</script>

<style scoped>
@keyframes fade-in {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}
.animate-fade-in {
  animation: fade-in 0.2s ease;
}
@media (max-width: 640px) {
  .rounded { border-radius: 0 !important; }
}
</style> 