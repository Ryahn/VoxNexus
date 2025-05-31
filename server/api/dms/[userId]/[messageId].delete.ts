import { defineEventHandler } from 'h3';
import { connectToDatabase } from '../../../utils/mongoose';
import { getUserFromEvent } from '../../../utils/auth';
import DirectMessage from '../../../models/DirectMessage';
import { removeMessageFromIndex } from '../../../utils/zincsearch';

export default defineEventHandler(async (event) => {
  await connectToDatabase();
  let userPayload;
  try {
    userPayload = getUserFromEvent(event);
  } catch (err) {
    return { status: 401, error: 'Unauthorized' };
  }

  const userId = event.context.params?.userId;
  const messageId = event.context.params?.messageId;
  if (!userId || !messageId) {
    return { status: 400, error: 'User ID and Message ID are required.' };
  }

  // Only allow the user to delete their own messages
  if (userPayload.id !== userId) {
    return { status: 403, error: 'You can only delete your own messages.' };
  }

  const message = await DirectMessage.findById(messageId);
  if (!message) {
    return { status: 404, error: 'Message not found.' };
  }
  if (message.from.toString() !== userId) {
    return { status: 403, error: 'You can only delete messages you sent.' };
  }

  await message.deleteOne();
  try {
    await removeMessageFromIndex(messageId);
  } catch (err) {
    console.error('Failed to remove message from ZincSearch:', err);
    // Do not block deletion if ZincSearch fails
  }
  return { status: 200, message: 'Message deleted successfully.' };
}); 