import { defineEventHandler } from 'h3';
import { connectToDatabase } from '../../../../utils/mongoose';
import { getUserFromEvent } from '../../../../utils/auth';
import Mute from '../../../../models/Mute';
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

  const canMute = await hasPermission(userPayload.id, serverId, null, 'MUTE_MEMBERS');
  if (!canMute) {
    return { status: 403, error: 'Insufficient permissions.' };
  }

  const mute = await Mute.findOneAndDelete({ serverId, userId }).lean();
  if (!mute) {
    return { status: 404, error: 'Mute not found.' };
  }

  await logAudit({
    serverId,
    action: 'unmute',
    actor: userPayload.id,
    target: userId,
    targetType: 'user',
    details: {},
  });

  return { status: 200, message: 'User unmuted successfully.' };
}); 