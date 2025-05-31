import { defineEventHandler, readBody } from 'h3';
import { connectToDatabase } from '../../../utils/mongoose';
import { getUserFromEvent } from '../../../utils/auth';
import Channel from '../../../models/Channel';
import ChannelMessage from '../../../models/ChannelMessage';
import { indexMessage } from '../../../utils/zincsearch';

export default defineEventHandler(async (event) => {
  await connectToDatabase();
  let userPayload;
  try {
    userPayload = getUserFromEvent(event);
  } catch (err) {
    return { status: 401, error: 'Unauthorized' };
  }

  const channelId = event.context.params?.id;
  if (!channelId) {
    return { status: 400, error: 'Channel ID is required.' };
  }

  const channel = await Channel.findById(channelId);
  if (!channel) {
    return { status: 404, error: 'Channel not found.' };
  }

  const body = await readBody(event);
  const { content } = body;
  if (!content || typeof content !== 'string' || !content.trim()) {
    return { status: 400, error: 'Message content required.' };
  }

  const message = await ChannelMessage.create({
    channelId: channel._id,
    serverId: channel.serverId,
    authorId: userPayload.id,
    content,
    createdAt: new Date(),
    updatedAt: new Date(),
  });

  try {
    await indexMessage({
      id: message._id.toString(),
      content: message.content,
      authorId: message.authorId.toString(),
      channelId: message.channelId.toString(),
      createdAt: message.createdAt.toISOString(),
    });
  } catch (err) {
    console.error('Failed to index message in ZincSearch:', err);
    // Do not block creation if ZincSearch fails
  }

  return { status: 201, message };
}); 