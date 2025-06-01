// @ts-ignore
import { defineEventHandler, readMultipartFormData, H3Event } from 'h3';
import { connectToDatabase } from '../../../utils/mongoose';
import { getUserFromEvent } from '../../../utils/auth';

export default defineEventHandler(async (event: H3Event) => {
  await connectToDatabase();
  let userPayload;
  try {
    userPayload = getUserFromEvent(event);
  } catch (err) {
    return { status: 401, error: 'Unauthorized' };
  }

  const channelId = event.context.params?.id;
  if (!channelId) {
    return { status: 400, error: 'Channel ID required.' };
  }

  const form = await readMultipartFormData(event);
  if (!form || !form.length) {
    return { status: 400, error: 'No file uploaded.' };
  }
  const file = form.find((f: any) => f.name === 'attachment');
  if (!file || !file.data) {
    return { status: 400, error: 'No attachment file provided.' };
  }

  // Use AttachmentService
  // @ts-ignore
  const attachmentService = event.context.attachmentService as import('../../utils/attachment-service/AttachmentService').default;
  const attachmentPath = await attachmentService.uploadChannelAttachment(file.data, channelId, file.filename || 'file', file.type || 'application/octet-stream');
  const publicUrl = `/${attachmentPath}`;

  return { status: 200, attachmentUrl: publicUrl };
}); 