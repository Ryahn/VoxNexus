import { defineEventHandler } from 'h3';
import { connectToDatabase } from '../../../utils/mongoose';
import { getUserFromEvent } from '../../../utils/auth';
import Mute from '../../../models/Mute';
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

  const canMute = await hasPermission(userPayload.id, serverId, null, 'MUTE_MEMBERS');
  if (!canMute) {
    return { status: 403, error: 'Insufficient permissions.' };
  }

  const mutes = await Mute.find({ serverId }).sort({ createdAt: -1 }).lean();
  return { status: 200, mutes };
}); 