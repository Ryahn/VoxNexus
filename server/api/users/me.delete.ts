import { defineEventHandler } from 'h3';
import { connectToDatabase } from '../../utils/mongoose';
import { getUserFromEvent } from '../../utils/auth';
import User from '../../models/User';
import { removeUserFromIndex } from '../../utils/zincsearch';

export default defineEventHandler(async (event) => {
  await connectToDatabase();
  let userPayload;
  try {
    userPayload = getUserFromEvent(event);
  } catch (err) {
    return { status: 401, error: 'Unauthorized' };
  }

  const user = await User.findById(userPayload.id);
  if (!user) {
    return { status: 404, error: 'User not found.' };
  }

  await user.deleteOne();
  try {
    await removeUserFromIndex(user._id.toString());
  } catch (err) {
    console.error('Failed to remove user from ZincSearch:', err);
    // Do not block deletion if ZincSearch fails
  }
  return { status: 200, message: 'User account deleted successfully.' };
}); 