import { defineEventHandler, readBody } from 'h3';
import { connectToDatabase } from '../../../../utils/mongoose';
import { getUserFromEvent } from '../../../../utils/auth';
import Role from '../../../../models/Role';
import { hasPermission } from '../../../../services/permissionService';
import { logRoleUpdate } from '../../../../services/auditLogService';

export default defineEventHandler(async (event) => {
  await connectToDatabase();
  let userPayload;
  try {
    userPayload = getUserFromEvent(event);
  } catch (err) {
    return { status: 401, error: 'Unauthorized' };
  }

  const serverId = event.context.params?.serverId;
  const roleId = event.context.params?.roleId;
  if (!serverId || !roleId) {
    return { status: 400, error: 'Server ID and Role ID are required.' };
  }

  const canManageRoles = await hasPermission(userPayload.id, serverId, null, 'MANAGE_ROLES');
  if (!canManageRoles) {
    return { status: 403, error: 'Insufficient permissions.' };
  }

  const body = await readBody(event);
  const { name, priority, permissions, color } = body;
  if (!name || typeof priority !== 'number' || !Array.isArray(permissions)) {
    return { status: 400, error: 'Invalid role data.' };
  }

  const role = await Role.findOneAndUpdate(
    { _id: roleId, serverId },
    { $set: { name, priority, permissions, color } },
    { new: true }
  ).lean();
  if (!role) {
    return { status: 404, error: 'Role not found.' };
  }

  await logRoleUpdate(serverId, userPayload.id, roleId, { name, priority, permissions, color });

  return { status: 200, role };
}); 