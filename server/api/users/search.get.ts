import { defineEventHandler, getQuery } from 'h3';
import { connectToDatabase } from '../../utils/mongoose';
import { getUserFromEvent } from '../../utils/auth';
import { rateLimit } from '../../utils/rateLimiter';
import { searchUsers } from '../../utils/zincsearch';

export default defineEventHandler(async (event) => {
  await rateLimit(event, { key: 'usersearch', window: 60, max: 60 });
  await connectToDatabase();
  let userPayload;
  try {
    userPayload = getUserFromEvent(event);
  } catch (err) {
    return { status: 401, error: 'Unauthorized' };
  }

  const { q } = getQuery(event);
  if (!q || typeof q !== 'string' || q.length < 2) {
    return { status: 400, error: 'Query too short.' };
  }

  const users = await searchUsers(q, 20);

  return { status: 200, users };
}); 