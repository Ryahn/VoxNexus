import { defineEventHandler, readBody } from 'h3';
import { connectToDatabase } from '../../utils/mongoose';
import { getUserFromEvent } from '../../utils/auth';
import FriendRequest from '../../models/FriendRequest';
import { emitToUser } from '../../socket/index';

export default defineEventHandler(async (event) => {
  await connectToDatabase();
  let userPayload;
  try {
    userPayload = getUserFromEvent(event);
  } catch (err) {
    return { status: 401, error: 'Unauthorized' };
  }

  const body = await readBody(event);
  const { requestId } = body;
  if (!requestId) {
    return { status: 400, error: 'Request ID required.' };
  }

  const request = await FriendRequest.findById(requestId);
  if (!request) {
    return { status: 404, error: 'Request not found.' };
  }
  if (request.to.toString() !== userPayload.id) {
    return { status: 403, error: 'Not authorized.' };
  }
  if (request.status !== 'pending') {
    return { status: 409, error: 'Request already handled.' };
  }

  request.status = 'rejected';
  await request.save();

  // Emit real-time event to sender
  emitToUser(request.from.toString(), 'friend:request:rejected', { from: userPayload.id, username: userPayload.username });

  return { status: 200, request };
}); 