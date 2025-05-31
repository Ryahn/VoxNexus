import { defineEventHandler, readBody } from 'h3';
import { connectToDatabase } from '../../../../utils/mongoose';
import { getUserFromEvent } from '../../../../utils/auth';
import ChannelMessage, { IChannelMessage } from '../../../../models/ChannelMessage';

export default defineEventHandler(async (event) => {
  await connectToDatabase();
  let userPayload;
  try {
    userPayload = getUserFromEvent(event);
  } catch (err) {
    return { status: 401, error: 'Unauthorized' };
  }

  const channelId = event.context.params?.id;
  const messageId = event.context.params?.messageId;
  if (!channelId || !messageId) {
    return { status: 400, error: 'Channel ID and Message ID are required.' };
  }

  const body = await readBody(event);
  const { emoji, action } = body;
  if (!emoji || !['add', 'remove'].includes(action)) {
    return { status: 400, error: 'Invalid emoji or action.' };
  }

  const message = await ChannelMessage.findById(messageId) as IChannelMessage | null;
  if (!message) {
    return { status: 404, error: 'Message not found.' };
  }
  if (message.channelId.toString() !== channelId) {
    return { status: 400, error: 'Message does not belong to this channel.' };
  }

  let updated = false;
  if (!message.reactions) message.reactions = [];
  const reaction = message.reactions.find((r: { emoji: string; userIds: any[] }) => r.emoji === emoji);
  if (action === 'add') {
    if (reaction) {
      if (!reaction.userIds.map(String).includes(userPayload.id)) {
        reaction.userIds.push(userPayload.id);
        updated = true;
      }
    } else {
      message.reactions.push({ emoji, userIds: [userPayload.id] });
      updated = true;
    }
  } else if (action === 'remove') {
    if (reaction) {
      const idx = reaction.userIds.map(String).indexOf(userPayload.id);
      if (idx !== -1) {
        reaction.userIds.splice(idx, 1);
        updated = true;
        // Remove reaction if no users left
        if (reaction.userIds.length === 0) {
          message.reactions = message.reactions.filter((r: { emoji: string; userIds: any[] }) => r.emoji !== emoji);
        }
      }
    }
  }
  if (updated) {
    await message.save();
  }
  return { status: 200, reactions: message.reactions };
}); 