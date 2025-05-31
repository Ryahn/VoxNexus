import { defineEventHandler, readBody } from 'h3';
import { connectToDatabase } from '../../utils/mongoose';
import { getUserFromEvent } from '../../utils/auth';
import User from '../../models/User';
import DirectMessage from '../../models/DirectMessage';
import { rateLimit } from '../../utils/rateLimiter';

export default defineEventHandler(async (event) => {
  await rateLimit(event, { key: 'dm_send', window: 60, max: 60 });
  await connectToDatabase();
  let userPayload;
  try {
    userPayload = getUserFromEvent(event);
  } catch (err) {
    return { status: 401, error: 'Unauthorized' };
  }

  const toUserId = event.context.params?.userId;
  if (!toUserId || toUserId === userPayload.id) {
    return { status: 400, error: 'Invalid user.' };
  }

  const toUser = await User.findById(toUserId);
  if (!toUser) {
    return { status: 404, error: 'User not found.' };
  }

  const body = await readBody(event);
  const { content } = body;
  if (!content || typeof content !== 'string' || !content.trim()) {
    return { status: 400, error: 'Message content required.' };
  }

  const message = await DirectMessage.create({
    from: userPayload.id,
    to: toUserId,
    content,
  });

  // Real-time event will be handled in socket server
  return { status: 201, message };
}); 