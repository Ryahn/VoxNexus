import { defineEventHandler, readBody } from 'h3';
import { connectToDatabase } from '../../../utils/mongoose';
import { getUserFromEvent } from '../../../utils/auth';
import Channel from '../../../models/Channel';
import ChannelMessage from '../../../models/ChannelMessage';
import { removeMessageFromIndex } from '../../../utils/zincsearch';
import { hasPermission } from '../../../services/permissionService';
import { logAudit } from '../../../services/auditLogService';

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

  const canManage = await hasPermission(userPayload.id, channel.serverId.toString(), channelId, 'MANAGE_MESSAGES');
  if (!canManage) {
    return { status: 403, error: 'Insufficient permissions.' };
  }

  const body = await readBody(event);
  const { messageIds } = body;
  if (!Array.isArray(messageIds) || !messageIds.length) {
    return { status: 400, error: 'messageIds must be a non-empty array.' };
  }

  const result = await ChannelMessage.deleteMany({ _id: { $in: messageIds }, channelId });
  for (const id of messageIds) {
    try { await removeMessageFromIndex(id); } catch {}
  }

  await logAudit({
    serverId: channel.serverId.toString(),
    action: 'bulk_delete_messages',
    actor: userPayload.id,
    target: channelId,
    targetType: 'channel',
    details: { messageIds },
  });

  return { status: 200, deleted: result.deletedCount };
});
