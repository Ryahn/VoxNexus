import { defineEventHandler, getQuery } from 'h3';
import { connectToDatabase } from '../utils/mongoose';
import { getUserFromEvent } from '../utils/auth';
import { hasPermission } from '../services/permissionService';
import type { PermissionKey } from '../types/permissions';

export default defineEventHandler(async (event) => {
  await connectToDatabase();
  let userPayload;
  try {
    userPayload = getUserFromEvent(event);
  } catch (err) {
    return { status: 401, error: 'Unauthorized' };
  }

  const { userId, serverId, channelId, permission } = getQuery(event);
  if (!userId || !serverId || !permission) {
    return { status: 400, error: 'userId, serverId, and permission are required.' };
  }

  const allowed = await hasPermission(
    String(userId),
    String(serverId),
    channelId ? String(channelId) : null,
    permission as PermissionKey
  );
  return { status: 200, hasPermission: allowed };
}); 