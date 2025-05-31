import { defineEventHandler, getQuery } from 'h3';
import { connectToDatabase } from '../../../utils/mongoose';
import { getUserFromEvent } from '../../../utils/auth';
import User from '../../../models/User';
import { searchDirectMessages } from '../../../utils/zincsearch';

export default defineEventHandler(async (event) => {
  await connectToDatabase();
  let userPayload;
  try {
    userPayload = getUserFromEvent(event);
  } catch (err) {
    return { status: 401, error: 'Unauthorized' };
  }

  const otherUserId = event.context.params?.userId;
  if (!otherUserId) {
    return { status: 400, error: 'User ID is required.' };
  }

  const otherUser = await User.findById(otherUserId);
  if (!otherUser) {
    return { status: 404, error: 'User not found.' };
  }

  const { q, limit, after } = getQuery(event);
  if (!q || typeof q !== 'string' || q.length < 2) {
    return { status: 400, error: 'Query too short.' };
  }
  const pageLimit = Math.min(Number(limit) || 20, 100);

  let messages = [];
  try {
    messages = await searchDirectMessages(q, userPayload.id, otherUserId, pageLimit + 1);
  } catch (err) {
    console.error('ZincSearch error:', err);
    return { status: 500, error: 'Search failed.' };
  }

  let startIndex = 0;
  if (after) {
    startIndex = messages.findIndex((msg) => msg._id === after) + 1;
    if (startIndex < 1) startIndex = 0;
  }
  const pagedMessages = messages.slice(startIndex, startIndex + pageLimit);
  let nextCursor = null;
  if (messages.length > startIndex + pageLimit) {
    nextCursor = pagedMessages[pagedMessages.length - 1]?._id;
  }

  return { status: 200, messages: pagedMessages, nextCursor };
}); 