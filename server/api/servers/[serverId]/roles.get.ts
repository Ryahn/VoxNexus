import { defineEventHandler } from 'h3';
import { connectToDatabase } from '../../../utils/mongoose';
import { getUserFromEvent } from '../../../utils/auth';
import Role from '../../../models/Role';
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

  const canManageRoles = await hasPermission(userPayload.id, serverId, null, 'MANAGE_ROLES');
  if (!canManageRoles) {
    return { status: 403, error: 'Insufficient permissions.' };
  }

  const roles = await Role.find({ serverId }).sort({ priority: -1 }).lean();
  return { status: 200, roles };
}); 