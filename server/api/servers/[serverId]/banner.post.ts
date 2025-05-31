import { defineEventHandler, readMultipartFormData } from 'h3';
import { connectToDatabase } from '../../../utils/mongoose';
import { getUserFromEvent } from '../../../utils/auth';
import Server from '../../../models/Server';

export default defineEventHandler(async (event) => {
  await connectToDatabase();
  let userPayload;
  try {
    userPayload = getUserFromEvent(event);
  } catch (err) {
    return { status: 401, error: 'Unauthorized' };
  }

  const serverId = event.context.params?.serverId;
  if (!serverId) {
    return { status: 400, error: 'Server ID required.' };
  }
  const server = await Server.findById(serverId);
  if (!server) {
    return { status: 404, error: 'Server not found.' };
  }
  if (server.ownerId.toString() !== userPayload.id) {
    return { status: 403, error: 'Only the server owner can update the banner.' };
  }

  const form = await readMultipartFormData(event);
  if (!form || !form.length) {
    return { status: 400, error: 'No file uploaded.' };
  }
  const file = form.find(f => f.name === 'banner');
  if (!file || !file.data) {
    return { status: 400, error: 'No banner file provided.' };
  }

  // Use AttachmentService
  // @ts-ignore
  const attachmentService = event.context.attachmentService as import('../../../utils/attachment-service/AttachmentService').default;
  const bannerPath = await attachmentService.uploadServerBanner(file.data, serverId, file.type || 'image/png');
  const publicUrl = `/${bannerPath}`;

  server.bannerUrl = publicUrl;
  await server.save();

  return { status: 200, bannerUrl: publicUrl };
}); 