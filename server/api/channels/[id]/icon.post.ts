import { defineEventHandler, readMultipartFormData } from 'h3';
import { connectToDatabase } from '../../utils/mongoose';
import { getUserFromEvent } from '../../utils/auth';
import Channel from '../../models/Channel';
import Server from '../../models/Server';

export default defineEventHandler(async (event) => {
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
  const channel = await Channel.findById(channelId);
  if (!channel) {
    return { status: 404, error: 'Channel not found.' };
  }
  const server = await Server.findById(channel.serverId);
  if (!server) {
    return { status: 404, error: 'Server not found.' };
  }
  if (server.ownerId.toString() !== userPayload.id) {
    return { status: 403, error: 'Only the server owner can update the channel icon.' };
  }

  const form = await readMultipartFormData(event);
  if (!form || !form.length) {
    return { status: 400, error: 'No file uploaded.' };
  }
  const file = form.find(f => f.name === 'icon');
  if (!file || !file.data) {
    return { status: 400, error: 'No icon file provided.' };
  }

  // Use AttachmentService
  // @ts-ignore
  const attachmentService = event.context.attachmentService as import('../../utils/attachment-service/AttachmentService').default;
  const iconPath = await attachmentService.uploadChannelIcon(file.data, channelId, file.type || 'image/png');
  const publicUrl = `/${iconPath}`;

  channel.iconUrl = publicUrl;
  await channel.save();

  return { status: 200, iconUrl: publicUrl };
}); 