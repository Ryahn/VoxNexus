<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useGroupDMChat } from '~/composables/useGroupDMChat';
import { useGroupDMSocket } from '~/composables/useGroupDMSocket';
import { useUserStore } from '~/store/user-store';
import { useGroupDMStore } from '~/store/group-dm-store';
import type { GroupDMMessagePayload, GroupDMMessageEditPayload, GroupDMMessageDeletePayload, GroupDMMessageReactionPayload } from '~/types/socket-events';

const route = useRoute();
const router = useRouter();
const groupId = computed(() => route.params.groupId as string);
const userStore = useUserStore();
const token = userStore.token;
const userId = userStore.user?.id;
const groupDMStore = useGroupDMStore();
const group = computed(() => groupDMStore.groups.find(g => g._id === groupId.value));

const { messages, fetchMessages, sendMessage } = useGroupDMChat(groupId.value);
const input = ref('');
const typing = ref(false);
const typingUsers = ref<string[]>([]);
const showEdit = ref(false);
const editName = ref('');
const editAvatar = ref('');

const {
  sendMessage: sendSocketMessage,
  sendTyping,
  sendStopTyping,
  onMessage,
  onTyping,
  onStopTyping,
  editMessage: socketEditMessage,
  deleteMessage: socketDeleteMessage,
  reactToMessage: socketReactToMessage,
  onEditMessage,
  onDeleteMessage,
  onReaction,
} = useGroupDMSocket(groupId.value, token);

const editingMessageId = ref<string | null>(null);
const editInput = ref('');
const showEmojiPicker = ref<string | null>(null);

// Tooltip state
const tooltipReaction = ref<string | null>(null);
const tooltipX = ref(0);
const tooltipY = ref(0);
const tooltipUsers = ref<string[]>([]);

function getUser(id: string) {
  return userStore.allUsers.find(u => u.id === id);
}

function getUsernames(userIds: string[]): string[] {
  return userIds
    .map(id => userStore.allUsers.find(u => u.id === id)?.username || 'Unknown')
    .filter(Boolean);
}

function handleSend() {
  if (!input.value.trim()) return;
  const msg: GroupDMMessagePayload = {
    groupId: groupId.value,
    authorId: userId,
    content: input.value,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  };
  sendMessage(input.value);
  sendSocketMessage(msg);
  input.value = '';
  sendStopTyping();
  groupDMStore.markAsRead(groupId.value);
}

function handleInput() {
  if (!typing.value) {
    sendTyping();
    typing.value = true;
    setTimeout(() => {
      typing.value = false;
      sendStopTyping();
    }, 3000);
  }
}

function handleEdit() {
  editName.value = group.value?.name || '';
  editAvatar.value = group.value?.avatarUrl || '';
  showEdit.value = true;
}
async function handleSaveEdit() {
  await groupDMStore.updateGroup(groupId.value, { name: editName.value, avatarUrl: editAvatar.value });
  showEdit.value = false;
}
async function handleLeave() {
  await groupDMStore.leaveOrDeleteGroup(groupId.value);
  router.push('/');
}

function startEdit(msgId: string, content: string) {
  editingMessageId.value = msgId;
  editInput.value = content;
}
function cancelEdit() {
  editingMessageId.value = null;
  editInput.value = '';
}
function confirmEdit(msg: any) {
  if (!editInput.value.trim()) return;
  const payload: GroupDMMessageEditPayload = {
    groupId: groupId.value,
    messageId: msg._id,
    content: editInput.value,
    updatedAt: new Date().toISOString(),
  };
  socketEditMessage(payload);
  // Optimistically update UI
  const m = messages.value.find((m: any) => m._id === msg._id);
  if (m) {
    m.content = editInput.value;
    m.updatedAt = payload.updatedAt;
  }
  cancelEdit();
}
function handleDelete(msg: any) {
  const payload: GroupDMMessageDeletePayload = {
    groupId: groupId.value,
    messageId: msg._id,
  };
  socketDeleteMessage(payload);
  // Optimistically remove from UI
  messages.value = messages.value.filter((m: any) => m._id !== msg._id);
}
function handleReact(msg: any, emoji: string) {
  const reaction = (msg.reactions || []).find((r: any) => r.emoji === emoji);
  const hasReacted = reaction && reaction.userIds.includes(userId);
  const payload: GroupDMMessageReactionPayload = {
    groupId: groupId.value,
    messageId: msg._id,
    emoji,
    userId: userId!,
    action: hasReacted ? 'remove' : 'add',
  };
  socketReactToMessage(payload);
}

function showTooltip(event: MouseEvent, reaction: any) {
  tooltipReaction.value = reaction.emoji;
  tooltipUsers.value = getUsernames(reaction.userIds);
  tooltipX.value = event.clientX;
  tooltipY.value = event.clientY;
}
function hideTooltip() {
  tooltipReaction.value = null;
  tooltipUsers.value = [];
}

// Animation state for reactions
const animatedReactions = ref<{ [emoji: string]: boolean }>({});
watch(
  () => messages.value.map(m => (m.reactions || []).map(r => `${m._id}:${r.emoji}:${r.userIds.length}`).join(',')).join(','),
  () => {
    messages.value.forEach(msg => {
      (msg.reactions || []).forEach(r => {
        animatedReactions.value[`${msg._id}:${r.emoji}`] = true;
        setTimeout(() => (animatedReactions.value[`${msg._id}:${r.emoji}`] = false), 300);
      });
    });
  }
);

onMounted(() => {
  fetchMessages();
  groupDMStore.markAsRead(groupId.value);
  userStore.fetchAllUsers();
  onMessage((msg) => {
    if (msg.authorId !== userId) {
      fetchMessages();
      if (route.params.groupId !== msg.groupId) {
        groupDMStore.incrementUnread(msg.groupId);
      }
    }
  });
  onTyping(({ from }) => {
    if (!typingUsers.value.includes(from) && from !== userId) typingUsers.value.push(from);
  });
  onStopTyping(({ from }) => {
    typingUsers.value = typingUsers.value.filter(u => u !== from);
  });
  onEditMessage((payload) => {
    const m = messages.value.find((m: any) => m._id === payload.messageId);
    if (m) {
      m.content = payload.content;
      m.updatedAt = payload.updatedAt;
    }
  });
  onDeleteMessage((payload) => {
    messages.value = messages.value.filter((m: any) => m._id !== payload.messageId);
  });
  onReaction((payload) => {
    const m = messages.value.find((m: any) => m._id === payload.messageId);
    if (m) {
      if (!m.reactions) m.reactions = [];
      let reaction = m.reactions.find((r: any) => r.emoji === payload.emoji);
      if (payload.action === 'add') {
        if (!reaction) {
          reaction = { emoji: payload.emoji, userIds: [] };
          m.reactions.push(reaction);
        }
        if (!reaction.userIds.includes(payload.userId)) reaction.userIds.push(payload.userId);
      } else if (payload.action === 'remove' && reaction) {
        reaction.userIds = reaction.userIds.filter((id: string) => id !== payload.userId);
        if (reaction.userIds.length === 0) {
          m.reactions = m.reactions.filter((r: any) => r.emoji !== payload.emoji);
        }
      }
    }
  });
});
</script>

<template>
  <div class="flex flex-col h-full w-full bg-[var(--color-bg-main)] p-2 sm:p-0">
    <div class="flex items-center justify-between px-2 sm:px-4 py-2 bg-[var(--color-bg-secondary)] border-b border-[var(--color-border)]">
      <div class="flex items-center space-x-3">
        <img v-if="group?.avatarUrl" :src="group.avatarUrl" class="w-10 h-10 rounded-full object-cover" />
        <div>
          <div class="text-base sm:text-lg font-bold text-[var(--color-text-main)]">{{ group?.name || 'Group DM' }}</div>
          <div class="text-xs text-[var(--color-text-secondary)] flex items-center space-x-1">
            <span v-for="id in group?.memberIds || []" :key="id" class="flex items-center">
              <img v-if="getUser(id)?.avatarUrl" :src="getUser(id).avatarUrl" class="w-5 h-5 rounded-full object-cover inline-block mr-1" />
              <span class="text-[var(--color-text-main)]">{{ getUser(id)?.username || id }}</span>
            </span>
          </div>
        </div>
      </div>
      <div class="flex items-center space-x-2">
        <button @click="handleEdit" class="px-2 py-1 text-xs bg-[var(--color-bg-tertiary)] text-[var(--color-text-main)] rounded hover:bg-[var(--color-bg-input)] border border-[var(--color-border)]">Edit</button>
        <button @click="handleLeave" class="px-2 py-1 text-xs bg-[var(--color-danger)] text-white rounded hover:bg-red-700">Leave</button>
      </div>
    </div>
    <div v-if="showEdit" class="p-4 bg-[var(--color-bg-main)] border-b border-[var(--color-border)] flex items-center space-x-2">
      <input v-model="editName" class="rounded bg-[var(--color-bg-input)] text-[var(--color-text-main)] px-3 py-2 focus:outline-none border border-[var(--color-border)]" placeholder="Group Name" />
      <input v-model="editAvatar" class="rounded bg-[var(--color-bg-input)] text-[var(--color-text-main)] px-3 py-2 focus:outline-none border border-[var(--color-border)]" placeholder="Avatar URL" />
      <button @click="handleSaveEdit" class="px-3 py-2 bg-[var(--color-accent)] text-white rounded hover:bg-[var(--color-accent-hover)]">Save</button>
      <button @click="showEdit=false" class="px-3 py-2 bg-[var(--color-bg-tertiary)] text-[var(--color-text-main)] rounded hover:bg-[var(--color-bg-input)] border border-[var(--color-border)]">Cancel</button>
    </div>
    <div class="flex-1 overflow-y-auto p-2 sm:p-4 space-y-2">
      <div
        v-for="msg in messages"
        :key="msg._id"
        class="flex items-end space-x-2"
        :class="msg.authorId === userId ? 'justify-end' : 'justify-start'"
      >
        <div v-if="msg.authorId !== userId" class="flex-shrink-0 w-8 h-8 rounded-full bg-[var(--color-bg-tertiary)] flex items-center justify-center">
          <img v-if="getUser(msg.authorId)?.avatarUrl" :src="getUser(msg.authorId).avatarUrl" class="w-8 h-8 rounded-full object-cover" />
          <span v-else class="text-[var(--color-text-main)] font-bold">{{ getUser(msg.authorId)?.username?.[0] || '?' }}</span>
        </div>
        <div :class="['max-w-xs', msg.authorId === userId ? 'bg-[var(--color-accent)] text-white ml-auto' : 'bg-[var(--color-bg-tertiary)] text-[var(--color-text-main)] border border-[var(--color-border)]']" class="rounded-lg px-2 sm:px-3 py-2 relative">
          <div class="text-sm font-semibold text-[var(--color-text-secondary)]" v-if="msg.authorId !== userId">{{ getUser(msg.authorId)?.username || msg.authorId }}</div>
          <div v-if="editingMessageId === msg._id">
            <input v-model="editInput" class="rounded bg-[var(--color-bg-input)] text-[var(--color-text-main)] px-2 py-1 w-full border border-[var(--color-border)]" />
            <div class="flex space-x-2 mt-1">
              <button @click="() => confirmEdit(msg)" class="px-2 py-1 bg-[var(--color-success)] text-white rounded">Save</button>
              <button @click="cancelEdit" class="px-2 py-1 bg-[var(--color-bg-tertiary)] text-[var(--color-text-main)] rounded border border-[var(--color-border)]">Cancel</button>
            </div>
          </div>
          <div v-else>
            <div class="text-base break-words">{{ msg.content }}</div>
            <div class="flex items-center space-x-1 mt-1">
              <span
                v-for="reaction in msg.reactions || []"
                :key="reaction.emoji"
                class="inline-flex items-center px-2 py-0.5 rounded-full text-xs cursor-pointer relative"
                :class="[
                  'bg-gray-700',
                  reaction.userIds.includes(userId) ? 'ring-2 ring-yellow-400 bg-yellow-100 text-yellow-900' : '',
                  animatedReactions[`${msg._id}:${reaction.emoji}`] ? 'scale-110 transition-transform duration-200' : ''
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
              <button @click="showEmojiPicker = msg._id" class="ml-1 text-yellow-400 hover:text-yellow-300">😊</button>
              <div v-if="showEmojiPicker === msg._id" class="absolute z-10 bg-gray-800 border border-gray-700 rounded p-2 mt-1">
                <button @click="() => { handleReact(msg, '👍'); showEmojiPicker = null; }" class="text-lg">👍</button>
                <button @click="() => { handleReact(msg, '😂'); showEmojiPicker = null; }" class="text-lg">😂</button>
                <button @click="() => { handleReact(msg, '❤️'); showEmojiPicker = null; }" class="text-lg">❤️</button>
                <button @click="showEmojiPicker = null" class="ml-2 text-gray-400">×</button>
              </div>
            </div>
          </div>
          <div class="text-xs text-[var(--color-text-secondary)]">{{ new Date(msg.createdAt).toLocaleTimeString() }}</div>
          <div v-if="msg.authorId === userId && editingMessageId !== msg._id" class="flex space-x-1 mt-1">
            <button @click="() => startEdit(msg._id, msg.content)" class="text-xs text-[var(--color-accent)] hover:text-[var(--color-accent-hover)]">Edit</button>
            <button @click="() => handleDelete(msg)" class="text-xs text-[var(--color-danger)] hover:text-red-500">Delete</button>
          </div>
        </div>
        <div v-if="msg.authorId === userId" class="flex-shrink-0 w-8 h-8 rounded-full bg-[var(--color-accent)] flex items-center justify-center">
          <img v-if="getUser(msg.authorId)?.avatarUrl" :src="getUser(msg.authorId).avatarUrl" class="w-8 h-8 rounded-full object-cover" />
          <span v-else class="text-white font-bold">{{ getUser(msg.authorId)?.username?.[0] || '?' }}</span>
        </div>
      </div>
      <div v-if="typingUsers.length" class="text-xs text-[var(--color-accent)] mt-2">{{ typingUsers.join(', ') }} typing...</div>
    </div>
    <form class="p-2 sm:p-4 flex items-center bg-[var(--color-bg-secondary)] sticky bottom-0 z-20 border-t border-[var(--color-border)]" @submit.prevent="handleSend">
      <input
        v-model="input"
        @input="handleInput"
        class="flex-1 rounded bg-[var(--color-bg-input)] text-[var(--color-text-main)] px-3 py-2 focus:outline-none text-base border border-[var(--color-border)]"
        placeholder="Type a message..."
        autocomplete="off"
      />
      <button type="submit" class="ml-2 px-4 py-2 bg-[var(--color-accent)] text-white rounded hover:bg-[var(--color-accent-hover)] text-base">Send</button>
    </form>
  </div>
</template>

<style scoped>
.flex-1 {
  flex: 1 1 0%;
}
.max-w-xs {
  max-width: 20rem;
}
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