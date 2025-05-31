import { defineEventHandler } from 'h3';
import { connectToDatabase } from '../../../utils/mongoose';
import { getUserFromEvent } from '../../../utils/auth';
import Ban from '../../../models/Ban';
import { hasPermission } from '../../../services/permissionService';

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

  const bans = await Ban.find({ serverId }).sort({ createdAt: -1 }).lean();
  return { status: 200, bans };
}); 