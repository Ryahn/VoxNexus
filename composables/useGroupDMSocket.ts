import { onMounted, onUnmounted } from 'vue';
import { io, Socket } from 'socket.io-client';
import type {
  GroupDMEvents,
  GroupDMMessagePayload,
  GroupDMTypingPayload,
  GroupDMMessageEditPayload,
  GroupDMMessageDeletePayload,
  GroupDMMessageReactionPayload
} from '~/types/socket-events';

let socket: Socket<GroupDMEvents>;

export function useGroupDMSocket(groupId: string, token: string) {
  if (!socket) {
    socket = io('/', { auth: { token } });
  }

  function joinGroupDM() {
    socket.emit('group-dm:join', groupId);
  }
  function leaveGroupDM() {
    socket.emit('group-dm:leave', groupId);
  }
  function sendMessage(msg: GroupDMMessagePayload) {
    socket.emit('group-dm:message', msg);
  }
  function sendTyping() {
    socket.emit('group-dm:typing', { groupId });
  }
  function sendStopTyping() {
    socket.emit('group-dm:stopTyping', { groupId });
  }
  function onMessage(cb: (msg: GroupDMMessagePayload) => void) {
    socket.on('group-dm:message', cb);
  }
  function onTyping(cb: (payload: { from: string }) => void) {
    socket.on('group-dm:typing', cb);
  }
  function onStopTyping(cb: (payload: { from: string }) => void) {
    socket.on('group-dm:stopTyping', cb);
  }
  function editMessage(payload: GroupDMMessageEditPayload) {
    socket.emit('group-dm:edit-message', payload);
  }
  function deleteMessage(payload: GroupDMMessageDeletePayload) {
    socket.emit('group-dm:delete-message', payload);
  }
  function reactToMessage(payload: GroupDMMessageReactionPayload) {
    socket.emit('group-dm:reaction', payload);
  }
  function onEditMessage(cb: (payload: GroupDMMessageEditPayload) => void) {
    socket.on('group-dm:edit-message', cb);
  }
  function onDeleteMessage(cb: (payload: GroupDMMessageDeletePayload) => void) {
    socket.on('group-dm:delete-message', cb);
  }
  function onReaction(cb: (payload: GroupDMMessageReactionPayload) => void) {
    socket.on('group-dm:reaction', cb);
  }

  onMounted(joinGroupDM);
  onUnmounted(leaveGroupDM);

  return {
    sendMessage,
    sendTyping,
    sendStopTyping,
    onMessage,
    onTyping,
    onStopTyping,
    editMessage,
    deleteMessage,
    reactToMessage,
    onEditMessage,
    onDeleteMessage,
    onReaction,
  };
} 