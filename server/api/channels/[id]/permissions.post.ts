import { defineEventHandler, readBody } from 'h3';
import { connectToDatabase } from '../../../utils/mongoose';
import { getUserFromEvent } from '../../../utils/auth';
import ChannelPermissionOverride from '../../../models/ChannelPermissionOverride';
import Channel from '../../../models/Channel';
import { hasPermission } from '../../../services/permissionService';
import { logPermissionOverride } from '../../../services/auditLogService';

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
    return { status: 400, error: 'Channel ID is required.' };
  }

  const channel = await Channel.findById(channelId);
  if (!channel) {
    return { status: 404, error: 'Channel not found.' };
  }

  const canManagePerms = await hasPermission(userPayload.id, channel.serverId.toString(), channelId, 'MANAGE_PERMISSIONS');
  if (!canManagePerms) {
    return { status: 403, error: 'Insufficient permissions.' };
  }

  const body = await readBody(event);
  const { roleId, userId, allow, deny } = body;
  if ((!roleId && !userId) || !Array.isArray(allow) || !Array.isArray(deny)) {
    return { status: 400, error: 'roleId or userId and allow/deny arrays are required.' };
  }

  const override = await ChannelPermissionOverride.findOneAndUpdate(
    { channelId, roleId: roleId || null, userId: userId || null },
    { $set: { allow, deny } },
    { new: true, upsert: true }
  ).lean();

  await logPermissionOverride(channel.serverId.toString(), userPayload.id, channelId, override._id.toString(), 'create', override);

  return { status: 200, override };
}); 