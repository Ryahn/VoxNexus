import { defineEventHandler, getQuery } from 'h3';
import { connectToDatabase } from '../../utils/mongoose';
import { getUserFromEvent } from '../../utils/auth';
import DirectMessage from '../../models/DirectMessage';

export default defineEventHandler(async (event) => {
  await connectToDatabase();
  let userPayload;
  try {
    userPayload = getUserFromEvent(event);
  } catch (err) {
    return { status: 401, error: 'Unauthorized' };
  }

  const userId = event.context.params?.userId;
  if (!userId) {
    return { status: 400, error: 'User ID required.' };
  }

  const { after, limit } = getQuery(event);
  const pageLimit = Math.min(Number(limit) || 50, 100);
  const query: any = {
    $or: [
      { from: userPayload.id, to: userId },
      { from: userId, to: userPayload.id },
    ],
  };
  if (after) {
    const afterMsg = await DirectMessage.findById(after).lean();
    if (afterMsg) {
      query.createdAt = { $gt: afterMsg.createdAt };
    }
  }

  const messages = await DirectMessage.find(query)
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