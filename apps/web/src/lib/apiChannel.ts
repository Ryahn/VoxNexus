import type { ChannelResponse } from '@voxnexus/api-client';
import type { Channel, ChannelType } from '../types';

const UI_TYPES: ChannelType[] = [
  'text',
  'announcement',
  'voice',
  'stream',
  'forum',
  'calendar',
  'events',
  'tasks',
  'docs',
  'media',
];

function isUiChannelType(value: string): value is ChannelType {
  return (UI_TYPES as string[]).includes(value);
}

/** Map API channel type strings to sidebar UI types. */
export function apiTypeToUi(type: string): ChannelType {
  switch (type) {
    case 'scheduling':
      return 'events';
    case 'streaming':
      return 'stream';
    case 'stage':
      return 'voice';
    default:
      return isUiChannelType(type) ? type : 'text';
  }
}

/** Map sidebar pickers to API channel type strings. */
export function uiTypeToApi(type: ChannelType): string {
  switch (type) {
    case 'events':
      return 'scheduling';
    case 'stream':
      return 'streaming';
    default:
      return type;
  }
}

export function apiChannelToUi(channel: ChannelResponse, spaceId: string): Channel {
  return {
    id: channel.id,
    groupId: spaceId,
    categoryId: channel.category_id ?? '',
    type: apiTypeToUi(channel.type),
    name: channel.name,
    topic: channel.topic || undefined,
  };
}
