import { defineEventHandler, getQuery } from 'h3';
import { connectToDatabase } from '../../../utils/mongoose';
import { getUserFromEvent } from '../../../utils/auth';
import Channel from '../../../models/Channel';
import ChannelMessage from '../../../models/ChannelMessage';

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

  const { after, limit } = getQuery(event);
  const pageLimit = Math.min(Number(limit) || 50, 100);
  const query: any = { channelId };
  if (after) {
    const afterMsg = await ChannelMessage.findById(after).lean();
    if (afterMsg) {
      query.createdAt = { $gt: afterMsg.createdAt };
    }
  }

  const messages = await ChannelMessage.find(query)
    .sort({ createdAt: 1 })
    .limit(pageLimit + 1)
    .lean();

  let nextCursor = null;
  if (messages.length > pageLimit) {
    nextCursor = messages[pageLimit]._id.toString();
    messages.length = pageLimit;
  }

  return { status: 200, messages, nextCursor };
}); 