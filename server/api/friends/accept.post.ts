import { defineEventHandler, readBody } from 'h3';
import { connectToDatabase } from '../../utils/mongoose';
import { getUserFromEvent } from '../../utils/auth';
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

  request.status = 'accepted';
  await request.save();

  return { status: 200, request };
}); 