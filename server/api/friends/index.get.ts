import { defineEventHandler } from 'h3';
import { connectToDatabase } from '../../utils/mongoose';
import { getUserFromEvent } from '../../utils/auth';
import FriendRequest from '../../models/FriendRequest';
import User from '../../models/User';

export default defineEventHandler(async (event) => {
  await connectToDatabase();
  let userPayload;
  try {
    userPayload = getUserFromEvent(event);
  } catch (err) {
    return { status: 401, error: 'Unauthorized' };
  }

  // Accepted friends (either direction)
  const accepted = await FriendRequest.find({
    $or: [
      { from: userPayload.id },
      { to: userPayload.id },
    ],
    status: 'accepted',
  }).lean();

  // Get friend user IDs
  const friendIds = accepted.map(r =>
    r.from.toString() === userPayload.id ? r.to : r.from
  );
  const friends = await User.find({ _id: { $in: friendIds } })
    .select('_id username avatarUrl bio status')
    .lean();

  // Incoming pending requests
  const incoming = await FriendRequest.find({
    to: userPayload.id,
    status: 'pending',
  })
    .populate('from', '_id username avatarUrl bio status')
    .lean();

  // Outgoing pending requests
  const outgoing = await FriendRequest.find({
    from: userPayload.id,
    status: 'pending',
  })
    .populate('to', '_id username avatarUrl bio status')
    .lean();

  return {
    status: 200,
    friends,
    incoming,
    outgoing,
  };
}); 