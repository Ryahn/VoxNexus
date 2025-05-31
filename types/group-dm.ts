import type { Ref } from 'vue';

export interface IGroupDM {
  _id: string;
  name?: string;
  ownerId: string;
  memberIds: string[];
  avatarUrl?: string;
  lastMessageAt?: string;
  createdAt: string;
}

export interface IGroupDMMessage {
  _id: string;
  groupId: string;
  authorId: string;
  content: string;
  attachments?: string[];
  reactions?: { emoji: string; userIds: string[] }[];
  createdAt: string;
  updatedAt: string;
} 