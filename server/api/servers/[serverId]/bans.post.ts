import { defineEventHandler, readBody } from 'h3';
import { connectToDatabase } from '../../../utils/mongoose';
import { getUserFromEvent } from '../../../utils/auth';
import Ban from '../../../models/Ban';
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

  const serverId = event.context.params?.serverId;
  if (!serverId) {
    return { status: 400, error: 'Server ID is required.' };
  }

  const canBan = await hasPermission(userPayload.id, serverId, null, 'BAN_MEMBERS');
  if (!canBan) {
    return { status: 403, error: 'Insufficient permissions.' };
  }

  const body = await readBody(event);
  const { userId, reason, expiresAt } = body;
  if (!userId) {
    return { status: 400, error: 'userId is required.' };
  }

  const ban = await Ban.create({
    serverId,
    userId,
    bannedBy: userPayload.id,
    reason,
    expiresAt,
  });

  await logAudit({
    serverId,
    action: 'ban',
    actor: userPayload.id,
    target: userId,
    targetType: 'user',
    details: { reason, expiresAt },
  });

  return { status: 201, ban };
}); 