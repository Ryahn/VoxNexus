import { defineEventHandler, getQuery } from 'h3';
import { connectToDatabase } from '../../../utils/mongoose';
import { getUserFromEvent } from '../../../utils/auth';
import AuditLog from '../../../models/AuditLog';
import { hasPermission } from '../../../services/permissionService';
import { Types } from 'mongoose';

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

  const canView = await hasPermission(userPayload.id, serverId, null, 'MANAGE_SERVER');
  if (!canView) {
    return { status: 403, error: 'Insufficient permissions.' };
  }

  const { limit, action, after, actor, target, targetType } = getQuery(event);
  const pageLimit = Math.min(Number(limit) || 50, 100);
  const query: any = { serverId };
  if (action) query.action = action;
  if (actor) query.actor = new Types.ObjectId(actor);
  if (target) query.target = new Types.ObjectId(target);
  if (targetType) query.targetType = targetType;
  if (after) {
    const afterLog = await AuditLog.findById(after).lean();
    if (afterLog) {
      query.createdAt = { $lt: afterLog.createdAt };
    }
  }

  const logs = await AuditLog.find(query)
    .sort({ createdAt: -1 })
    .limit(pageLimit + 1)
    .lean();

  let nextCursor = null;
  if (logs.length > pageLimit) {
    nextCursor = logs[pageLimit]._id.toString();
    logs.length = pageLimit;
  }

  return { status: 200, logs, nextCursor };
}); 