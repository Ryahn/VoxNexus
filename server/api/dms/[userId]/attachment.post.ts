import { defineEventHandler, readMultipartFormData } from 'h3';
import { connectToDatabase } from '../../utils/mongoose';
import { getUserFromEvent } from '../../utils/auth';

export default defineEventHandler(async (event) => {
  await connectToDatabase();
  let userPayload;
  try {
    userPayload = getUserFromEvent(event);
  } catch (err) {
    return { status: 401, error: 'Unauthorized' };
  }

  const toUserId = event.context.params?.userId;
  if (!toUserId) {
    return { status: 400, error: 'User ID required.' };
  }

  const form = await readMultipartFormData(event);
  if (!form || !form.length) {
    return { status: 400, error: 'No file uploaded.' };
  }
  const file = form.find(f => f.name === 'attachment');
  if (!file || !file.data) {
    return { status: 400, error: 'No attachment file provided.' };
  }

  // Use AttachmentService (group chat method for now)
  // @ts-ignore
  const attachmentService = event.context.attachmentService as import('../../utils/attachment-service/AttachmentService').default;
  const attachmentPath = await attachmentService.uploadGroupChatAttachment(file.data, toUserId, file.filename || 'file', file.type || 'application/octet-stream');
  const publicUrl = `/${attachmentPath}`;

  return { status: 200, attachmentUrl: publicUrl };
}); 