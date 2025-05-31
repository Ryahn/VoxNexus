import { defineEventHandler, readBody } from 'h3';
import { connectToDatabase } from '../../utils/mongoose';
import { getUserFromEvent } from '../../utils/auth';
import Channel, { ChannelType } from '../../models/Channel';
import Server from '../../models/Server';
import { hasPermission } from '../../services/permissionService';

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
    return { status: 403, error: 'Only the server owner can update the channel.' };
  }

  const canManage = await hasPermission(user.id, channel.serverId.toString(), channelId, 'MANAGE_CHANNELS');
  if (!canManage) {
    return { status: 403, error: 'Insufficient permissions.' };
  }

  const body = await readBody(event);
  const { name, type } = body;
  if (!name && !type) {
    return { status: 400, error: 'At least one of name or type is required.' };
  }
  if (name) channel.name = name;
  if (type && (type === 'text' || type === 'voice')) channel.type = type;
  await channel.save();

  return { status: 200, channel: { id: channel._id, name: channel.name, type: channel.type, createdAt: channel.createdAt } };
}); 