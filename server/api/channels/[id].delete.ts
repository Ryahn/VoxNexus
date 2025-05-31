import { defineEventHandler } from 'h3';
import { connectToDatabase } from '../../utils/mongoose';
import { getUserFromEvent } from '../../utils/auth';
import Channel from '../../models/Channel';
import Server from '../../models/Server';

export default defineEventHandler(async (event) => {
  await connectToDatabase();
  let user;
  try {
    user = getUserFromEvent(event);
  } catch (err) {
    return { status: 401, error: 'Unauthorized' };
  }

  const channelId = event.context.params?.id;
  if (!channelId) {
    return { status: 400, error: 'Channel ID is required.' };
  }

  const channel = await Channel.findById(channelId);
  if (!channel) {
    return { status: 404, error: 'Channel not found.' };
  }

  const server = await Server.findById(channel.serverId);
  if (!server) {
    return { status: 404, error: 'Server not found.' };
  }
  if (server.ownerId.toString() !== user.id) {
    return { status: 403, error: 'Only the server owner can delete the channel.' };
  }

  await channel.deleteOne();
  return { status: 200, message: 'Channel deleted successfully.' };
});
