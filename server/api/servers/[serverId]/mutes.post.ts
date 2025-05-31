import { defineEventHandler, readBody } from 'h3';
import { connectToDatabase } from '../../../utils/mongoose';
import { getUserFromEvent } from '../../../utils/auth';
import Mute from '../../../models/Mute';
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

  const canMute = await hasPermission(userPayload.id, serverId, null, 'MUTE_MEMBERS');
  if (!canMute) {
    return { status: 403, error: 'Insufficient permissions.' };
  }

  const body = await readBody(event);
  const { userId, reason, expiresAt } = body;
  if (!userId) {
    return { status: 400, error: 'userId is required.' };
  }

  const mute = await Mute.create({
    serverId,
    userId,
    mutedBy: userPayload.id,
    reason,
    expiresAt,
  });

  await logAudit({
    serverId,
    action: 'mute',
    actor: userPayload.id,
    target: userId,
    targetType: 'user',
    details: { reason, expiresAt },
  });

  return { status: 201, mute };
}); 