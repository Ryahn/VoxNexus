<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
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

function getUser(id: string) {
  return userStore.allUsers.find(u => u.id === id);
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
  <div class="flex flex-col h-full w-full bg-gray-900">
    <div class="flex items-center justify-between px-4 py-2 bg-gray-800 border-b border-gray-700">
      <div class="flex items-center space-x-3">
        <img v-if="group?.avatarUrl" :src="group.avatarUrl" class="w-10 h-10 rounded-full object-cover" />
        <div>
          <div class="text-lg font-bold text-white">{{ group?.name || 'Group DM' }}</div>
          <div class="text-xs text-gray-400 flex items-center space-x-1">
            <span v-for="id in group?.memberIds || []" :key="id" class="flex items-center">
              <img v-if="getUser(id)?.avatarUrl" :src="getUser(id).avatarUrl" class="w-5 h-5 rounded-full object-cover inline-block mr-1" />
              <span class="text-gray-300">{{ getUser(id)?.username || id }}</span>
            </span>
          </div>
        </div>
      </div>
      <div class="flex items-center space-x-2">
        <button @click="handleEdit" class="px-2 py-1 text-xs bg-gray-700 text-white rounded hover:bg-gray-600">Edit</button>
        <button @click="handleLeave" class="px-2 py-1 text-xs bg-red-600 text-white rounded hover:bg-red-700">Leave</button>
      </div>
    </div>
    <div v-if="showEdit" class="p-4 bg-gray-900 border-b border-gray-700 flex items-center space-x-2">
      <input v-model="editName" class="rounded bg-gray-700 text-white px-3 py-2 focus:outline-none" placeholder="Group Name" />
      <input v-model="editAvatar" class="rounded bg-gray-700 text-white px-3 py-2 focus:outline-none" placeholder="Avatar URL" />
      <button @click="handleSaveEdit" class="px-3 py-2 bg-blue-600 text-white rounded hover:bg-blue-700">Save</button>
      <button @click="showEdit=false" class="px-3 py-2 bg-gray-600 text-white rounded hover:bg-gray-700">Cancel</button>
    </div>
    <div class="flex-1 overflow-y-auto p-4 space-y-2">
      <div
        v-for="msg in messages"
        :key="msg._id"
        class="flex items-end space-x-2"
        :class="msg.authorId === userId ? 'justify-end' : 'justify-start'"
      >
        <div v-if="msg.authorId !== userId" class="flex-shrink-0 w-8 h-8 rounded-full bg-gray-700 flex items-center justify-center">
          <img v-if="getUser(msg.authorId)?.avatarUrl" :src="getUser(msg.authorId).avatarUrl" class="w-8 h-8 rounded-full object-cover" />
          <span v-else class="text-white font-bold">{{ getUser(msg.authorId)?.username?.[0] || '?' }}</span>
        </div>
        <div :class="['max-w-xs', msg.authorId === userId ? 'bg-blue-600 text-white ml-auto' : 'bg-gray-800 text-gray-100']" class="rounded-lg px-3 py-2 relative">
          <div class="text-sm font-semibold" v-if="msg.authorId !== userId">{{ getUser(msg.authorId)?.username || msg.authorId }}</div>
          <div v-if="editingMessageId === msg._id">
            <input v-model="editInput" class="rounded bg-gray-700 text-white px-2 py-1 w-full" />
            <div class="flex space-x-2 mt-1">
              <button @click="() => confirmEdit(msg)" class="px-2 py-1 bg-green-600 text-white rounded">Save</button>
              <button @click="cancelEdit" class="px-2 py-1 bg-gray-600 text-white rounded">Cancel</button>
            </div>
          </div>
          <div v-else>
            <div class="text-base">{{ msg.content }}</div>
            <div class="flex items-center space-x-1 mt-1">
              <span
                v-for="reaction in msg.reactions || []"
                :key="reaction.emoji"
                class="inline-flex items-center px-2 py-0.5 rounded-full text-xs cursor-pointer"
                :class="[
                  'bg-gray-700',
                  reaction.userIds.includes(userId) ? 'ring-2 ring-yellow-400 bg-yellow-100 text-yellow-900' : ''
                ]"
                @click="handleReact(msg, reaction.emoji)"
              >
                {{ reaction.emoji }} {{ reaction.userIds.length }}
              </span>
              <button @click="showEmojiPicker = msg._id" class="ml-1 text-yellow-400 hover:text-yellow-300">😊</button>
              <div v-if="showEmojiPicker === msg._id" class="absolute z-10 bg-gray-800 border border-gray-700 rounded p-2 mt-1">
                <button @click="() => { handleReact(msg, '👍'); showEmojiPicker = null; }" class="text-lg">👍</button>
                <button @click="() => { handleReact(msg, '😂'); showEmojiPicker = null; }" class="text-lg">😂</button>
                <button @click="() => { handleReact(msg, '❤️'); showEmojiPicker = null; }" class="text-lg">❤️</button>
                <button @click="showEmojiPicker = null" class="ml-2 text-gray-400">×</button>
              </div>
            </div>
          </div>
          <div class="text-xs text-gray-300">{{ new Date(msg.createdAt).toLocaleTimeString() }}</div>
          <div v-if="msg.authorId === userId && editingMessageId !== msg._id" class="flex space-x-1 mt-1">
            <button @click="() => startEdit(msg._id, msg.content)" class="text-xs text-blue-200 hover:text-blue-400">Edit</button>
            <button @click="() => handleDelete(msg)" class="text-xs text-red-300 hover:text-red-500">Delete</button>
          </div>
        </div>
        <div v-if="msg.authorId === userId" class="flex-shrink-0 w-8 h-8 rounded-full bg-blue-700 flex items-center justify-center">
          <img v-if="getUser(msg.authorId)?.avatarUrl" :src="getUser(msg.authorId).avatarUrl" class="w-8 h-8 rounded-full object-cover" />
          <span v-else class="text-white font-bold">{{ getUser(msg.authorId)?.username?.[0] || '?' }}</span>
        </div>
      </div>
      <div v-if="typingUsers.length" class="text-xs text-blue-400 mt-2">{{ typingUsers.join(', ') }} typing...</div>
    </div>
    <form class="p-4 flex items-center bg-gray-800" @submit.prevent="handleSend">
      <input
        v-model="input"
        @input="handleInput"
        class="flex-1 rounded bg-gray-700 text-white px-3 py-2 focus:outline-none"
        placeholder="Type a message..."
        autocomplete="off"
      />
      <button type="submit" class="ml-2 px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700">Send</button>
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
</style> 