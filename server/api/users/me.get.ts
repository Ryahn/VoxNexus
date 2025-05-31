import { defineEventHandler } from 'h3';
import { connectToDatabase } from '../../utils/mongoose';
import { getUserFromEvent } from '../../utils/auth';
import User from '../../models/User';

export default defineEventHandler(async (event) => {
  await connectToDatabase();
  let userPayload;
  try {
    userPayload = getUserFromEvent(event);
  } catch (err) {
    return { status: 401, error: 'Unauthorized' };
  }

  const user = await User.findById(userPayload.id).select('_id username email createdAt').lean();
  if (!user) {
    return { status: 404, error: 'User not found.' };
  }

  return { status: 200, user };
}); 