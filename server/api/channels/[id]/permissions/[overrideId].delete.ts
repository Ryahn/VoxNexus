import { defineEventHandler } from 'h3';
import { connectToDatabase } from '../../../../utils/mongoose';
import { getUserFromEvent } from '../../../../utils/auth';
import ChannelPermissionOverride from '../../../../models/ChannelPermissionOverride';
import Channel from '../../../../models/Channel';
import { hasPermission } from '../../../../services/permissionService';
import { logPermissionOverride } from '../../../../services/auditLogService';

export default defineEventHandler(async (event) => {
  await connectToDatabase();
  let userPayload;
  try {
    userPayload = getUserFromEvent(event);
  } catch (err) {
    return { status: 401, error: 'Unauthorized' };
  }

  const channelId = event.context.params?.id;
  const overrideId = event.context.params?.overrideId;
  if (!channelId || !overrideId) {
    return { status: 400, error: 'Channel ID and Override ID are required.' };
  }

  const channel = await Channel.findById(channelId);
  if (!channel) {
    return { status: 404, error: 'Channel not found.' };
  }

  const canManagePerms = await hasPermission(userPayload.id, channel.serverId.toString(), channelId, 'MANAGE_PERMISSIONS');
  if (!canManagePerms) {
    return { status: 403, error: 'Insufficient permissions.' };
  }

  const override = await ChannelPermissionOverride.findOneAndDelete({ _id: overrideId, channelId }).lean();
  if (!override) {
    return { status: 404, error: 'Override not found.' };
  }

  await logPermissionOverride(channel.serverId.toString(), userPayload.id, channelId, overrideId, 'delete', override);

  return { status: 200, message: 'Override deleted successfully.' };
}); 