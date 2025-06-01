import { defineEventHandler } from 'h3';
import { connectToDatabase } from '../../../../../utils/mongoose';
import { getUserFromEvent } from '../../../../../utils/auth';
import ServerMember from '../../../../../models/ServerMember';
import { hasPermission } from '../../../../../services/permissionService';
import { logAudit } from '../../../../../services/auditLogService';

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

  const canKick = await hasPermission(userPayload.id, serverId, null, 'KICK_MEMBERS');
  if (!canKick) {
    return { status: 403, error: 'Insufficient permissions.' };
  }

  const member = await ServerMember.findOneAndDelete({ serverId, userId }).lean();
  if (!member) {
    return { status: 404, error: 'Member not found.' };
  }

  await logAudit({
    serverId,
    action: 'kick',
    actor: userPayload.id,
    target: userId,
    targetType: 'user',
    details: {},
  });

  return { status: 200, message: 'User kicked from server.' };
}); 