import { defineEventHandler, readBody } from 'h3';
import { connectToDatabase } from '../../utils/mongoose';
import { getUserFromEvent } from '../../utils/auth';
import User from '../../models/User';
import FriendRequest from '../../models/FriendRequest';

export default defineEventHandler(async (event) => {
  await connectToDatabase();
  let userPayload;
  try {
    userPayload = getUserFromEvent(event);
  } catch (err) {
    return { status: 401, error: 'Unauthorized' };
  }

  const body = await readBody(event);
  const { toUserId } = body;
  if (!toUserId || toUserId === userPayload.id) {
    return { status: 400, error: 'Invalid user.' };
  }

  const toUser = await User.findById(toUserId);
  if (!toUser) {
    return { status: 404, error: 'User not found.' };
  }

  const existing = await FriendRequest.findOne({
    from: userPayload.id,
    to: toUserId,
    status: { $in: ['pending', 'accepted'] },
  });
  if (existing) {
    return { status: 409, error: 'Friend request already sent or already friends.' };
  }

  const request = await FriendRequest.create({
    from: userPayload.id,
    to: toUserId,
    status: 'pending',
  });

  return { status: 201, request };
}); 