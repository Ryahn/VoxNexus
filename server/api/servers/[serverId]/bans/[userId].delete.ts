import { defineEventHandler } from 'h3';
import { connectToDatabase } from '../../../../utils/mongoose';
import { getUserFromEvent } from '../../../../utils/auth';
import Ban from '../../../../models/Ban';
import { hasPermission } from '../../../../services/permissionService';
import { logAudit } from '../../../../services/auditLogService';

export default defineEventHandler(async (event) => {
  await connectToDatabase();
  let userPayload;
  try {
    userPayload = getUserFromEvent(event);
  } catch (err) {
    return { status: 401, error: 'Unauthorized' };
  }

  const serverId = event.context.params?.serverId;
  const userId = event.context.params?.userId;
  if (!serverId || !userId) {
    return { status: 400, error: 'Server ID and User ID are required.' };
  }

  const canBan = await hasPermission(userPayload.id, serverId, null, 'BAN_MEMBERS');
  if (!canBan) {
    return { status: 403, error: 'Insufficient permissions.' };
  }

  const ban = await Ban.findOneAndDelete({ serverId, userId }).lean();
  if (!ban) {
    return { status: 404, error: 'Ban not found.' };
  }

  await logAudit({
    serverId,
    action: 'unban',
    actor: userPayload.id,
    target: userId,
    targetType: 'user',
    details: {},
  });

  return { status: 200, message: 'User unbanned successfully.' };
}); 