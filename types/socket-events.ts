export interface GroupDMMessagePayload {
  groupId: string;
  authorId: string;
  content: string;
  attachments?: string[];
  reactions?: { emoji: string; userIds: string[] }[];
  createdAt: string;
  updatedAt: string;
}

export interface GroupDMTypingPayload {
  groupId: string;
}

export interface GroupDMMessageEditPayload {
  groupId: string;
  messageId: string;
  content: string;
  updatedAt: string;
}

export interface GroupDMMessageDeletePayload {
  groupId: string;
  messageId: string;
}

export interface GroupDMMessageReactionPayload {
  groupId: string;
  messageId: string;
  emoji: string;
  userId: string;
  action: 'add' | 'remove';
}

export interface GroupDMEvents {
  'group-dm:join': (groupId: string) => void;
  'group-dm:leave': (groupId: string) => void;
  'group-dm:message': (msg: GroupDMMessagePayload) => void;
  'group-dm:typing': (payload: GroupDMTypingPayload) => void;
  'group-dm:stopTyping': (payload: GroupDMTypingPayload) => void;
  'group-dm:edit-message': (payload: GroupDMMessageEditPayload) => void;
  'group-dm:delete-message': (payload: GroupDMMessageDeletePayload) => void;
  'group-dm:reaction': (payload: GroupDMMessageReactionPayload) => void;
}

export interface ChannelMessageEditPayload {
  channelId: string;
  messageId: string;
  content: string;
  updatedAt: string;
}

export interface ChannelMessageDeletePayload {
  channelId: string;
  messageId: string;
}

export interface ChannelMessageReactionPayload {
  channelId: string;
  messageId: string;
  emoji: string;
  userId: string;
  action: 'add' | 'remove';
}

export interface ChannelEvents {
  'channel:edit-message': (payload: ChannelMessageEditPayload) => void;
  'channel:delete-message': (payload: ChannelMessageDeletePayload) => void;
  'channel:reaction': (payload: ChannelMessageReactionPayload) => void;
} 