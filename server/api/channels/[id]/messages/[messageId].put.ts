import { defineEventHandler, readBody } from 'h3';
import { connectToDatabase } from '../../../../utils/mongoose';
import { getUserFromEvent } from '../../../../utils/auth';
import ChannelMessage, { IChannelMessage } from '../../../../models/ChannelMessage';
import { hasPermission } from '../../../../services/permissionService';

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
  const { content, attachments } = body;
  if (!content || typeof content !== 'string' || !content.trim()) {
    return { status: 400, error: 'Message content required.' };
  }

  const message = await ChannelMessage.findById(messageId) as IChannelMessage | null;
  if (!message) {
    return { status: 404, error: 'Message not found.' };
  }
  if (message.channelId.toString() !== channelId) {
    return { status: 400, error: 'Message does not belong to this channel.' };
  }
  if (message.authorId.toString() !== userPayload.id) {
    const canManage = await hasPermission(userPayload.id, message.serverId.toString(), channelId, 'MANAGE_MESSAGES');
    if (!canManage) {
      return { status: 403, error: 'You do not have permission to edit this message.' };
    }
  }

  message.content = content;
  if (attachments && Array.isArray(attachments)) {
    message.attachments = attachments;
  }
  message.updatedAt = new Date();
  await message.save();
  return { status: 200, message };
}); 