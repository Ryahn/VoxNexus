import { defineEventHandler, readBody } from 'h3';
import { connectToDatabase } from '../../../utils/mongoose';
import { getUserFromEvent } from '../../../utils/auth';
import ServerMember from '../../../models/ServerMember';
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
  const userId = event.context.params?.userId;
  if (!serverId || !userId) {
    return { status: 400, error: 'Server ID and User ID are required.' };
  }

  const canManageRoles = await hasPermission(userPayload.id, serverId, null, 'MANAGE_ROLES');
  if (!canManageRoles) {
    return { status: 403, error: 'Insufficient permissions.' };
  }

  const body = await readBody(event);
  const { roleIds } = body;
  if (!Array.isArray(roleIds)) {
    return { status: 400, error: 'roleIds must be an array.' };
  }

  const member = await ServerMember.findOneAndUpdate(
    { serverId, userId },
    { $set: { roleIds } },
    { new: true, upsert: true }
  ).lean();

  return { status: 200, member };
}); 