<template>
  <div class="flex flex-col items-center min-h-screen bg-nightgray p-8">
    <div class="w-full max-w-xl bg-gray-800 rounded shadow-md flex flex-col h-[80vh]">
      <div class="p-4 border-b border-gray-700 flex items-center">
        <img :src="otherUser.avatarUrl || 'https://ui-avatars.com/api/?name=' + otherUser.username" class="w-10 h-10 rounded-full mr-3 object-cover" alt="Avatar" />
        <div>
          <div class="text-white font-semibold">{{ otherUser.username }}</div>
          <div class="text-xs text-gray-400">{{ otherUser.status }}</div>
        </div>
      </div>
      <div class="flex-1 overflow-y-auto p-4 space-y-4">
        <div v-for="(msg, i) in messages" :key="i" :class="msg.from === myId ? 'text-right' : 'text-left'">
          <div class="inline-block max-w-[70%] p-2 rounded-lg" :class="msg.from === myId ? 'bg-green-600 text-white' : 'bg-gray-700 text-white'">
            <div class="text-xs text-gray-300 mb-1">{{ msg.from === myId ? 'You' : otherUser.username }}</div>
            <div>{{ msg.content }}</div>
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
            <div class="text-xs text-gray-400 mt-1">{{ msg.createdAt }}</div>
          </div>
        </div>
        <div v-if="isTyping" class="text-xs text-gray-400 mt-2">{{ otherUser.username }} is typing...</div>
      </div>
      <form @submit.prevent="handleSend" class="flex items-center p-4 border-t border-gray-700">
        <input v-model="input" @input="handleTyping" @blur="stopTyping" class="flex-1 p-2 rounded bg-gray-700 text-white mr-2" placeholder="Type a message..." />
        <button type="submit" class="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700">Send</button>
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
</script>

<style scoped>
@keyframes fade-in {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}
.animate-fade-in {
  animation: fade-in 0.2s ease;
}
</style> 