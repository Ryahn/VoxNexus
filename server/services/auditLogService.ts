import AuditLog from '../models/AuditLog';
import type { IAuditLog } from '../models/AuditLog';
import { Types } from 'mongoose';

interface LogAuditParams {
  serverId: string;
  action: string;
  actor: string;
  target?: string;
  targetType: string;
  details?: Record<string, any>;
}

export async function logAudit({ serverId, action, actor, target, targetType, details }: LogAuditParams): Promise<IAuditLog> {
  return AuditLog.create({
    serverId: new Types.ObjectId(serverId),
    action,
    actor: new Types.ObjectId(actor),
    target: target ? new Types.ObjectId(target) : undefined,
    targetType,
    details,
  });
}

// Helper event loggers
export async function logRoleCreate(serverId: string, actor: string, roleId: string, details?: Record<string, any>) {
  return logAudit({ serverId, action: 'role_create', actor, target: roleId, targetType: 'role', details });
}
export async function logRoleUpdate(serverId: string, actor: string, roleId: string, details?: Record<string, any>) {
  return logAudit({ serverId, action: 'role_update', actor, target: roleId, targetType: 'role', details });
}
export async function logRoleDelete(serverId: string, actor: string, roleId: string, details?: Record<string, any>) {
  return logAudit({ serverId, action: 'role_delete', actor, target: roleId, targetType: 'role', details });
}
export async function logPermissionOverride(serverId: string, actor: string, channelId: string, overrideId: string, action: 'create' | 'delete', details?: Record<string, any>) {
  return logAudit({ serverId, action: `permission_override_${action}`, actor, target: overrideId, targetType: 'permission_override', details: { channelId, ...details } });
}
export async function logChannelCreate(serverId: string, actor: string, channelId: string, details?: Record<string, any>) {
  return logAudit({ serverId, action: 'channel_create', actor, target: channelId, targetType: 'channel', details });
}
export async function logChannelDelete(serverId: string, actor: string, channelId: string, details?: Record<string, any>) {
  return logAudit({ serverId, action: 'channel_delete', actor, target: channelId, targetType: 'channel', details });
}
export async function logServerUpdate(serverId: string, actor: string, details?: Record<string, any>) {
  return logAudit({ serverId, action: 'server_update', actor, target: serverId, targetType: 'server', details });
} 