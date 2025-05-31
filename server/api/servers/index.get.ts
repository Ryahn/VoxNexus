import { defineEventHandler } from 'h3';
import { connectToDatabase } from '../../utils/mongoose';
import { getUserFromEvent } from '../../utils/auth';
import Server from '../../models/Server';

export default defineEventHandler(async (event) => {
  await connectToDatabase();
  let user;
  try {
    user = getUserFromEvent(event);
  } catch (err) {
    return { status: 401, error: 'Unauthorized' };
  }

  const servers = await Server.find({ members: user.id })
    .select('_id name ownerId createdAt')
    .lean();

  return { status: 200, servers };
}); 