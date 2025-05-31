import { PermissionKey, PERMISSIONS } from '../types/permissions';
import Role from '../models/Role';
import ServerMember from '../models/ServerMember';
import ChannelPermissionOverride from '../models/ChannelPermissionOverride';
import { Types } from 'mongoose';

export async function hasPermission(userId: string, serverId: string, channelId: string | null, permission: PermissionKey): Promise<boolean> {
  // Get member roles
  const member = await ServerMember.findOne({ serverId, userId }).lean();
  if (!member) return false;
  const roles = await Role.find({ _id: { $in: member.roleIds } }).lean();
  if (!roles.length) return false;
  // Sort by priority descending
  roles.sort((a, b) => b.priority - a.priority);
  // ADMINISTRATOR bypass
  if (roles.some(r => r.permissions.includes(PERMISSIONS.ADMINISTRATOR))) return true;
  // Aggregate permissions
  let allowed = new Set<string>();
  let denied = new Set<string>();
  for (const role of roles) {
    for (const perm of role.permissions) allowed.add(perm);
  }
  // Channel overrides
  if (channelId) {
    const overrides = await ChannelPermissionOverride.find({
      channelId,
      $or: [
        { roleId: { $in: member.roleIds } },
        { userId: new Types.ObjectId(userId) },
      ],
    }).lean();
    // Apply overrides: user-specific first, then roles (highest priority role wins)
    for (const override of overrides) {
      if (override.userId && override.userId.toString() === userId) {
        for (const perm of override.allow) allowed.add(perm);
        for (const perm of override.deny) denied.add(perm);
      }
    }
    for (const role of roles) {
      const roleOverride = overrides.find(o => o.roleId && o.roleId.toString() === role._id.toString());
      if (roleOverride) {
        for (const perm of roleOverride.allow) allowed.add(perm);
        for (const perm of roleOverride.deny) denied.add(perm);
      }
    }
  }
  // Deny takes precedence
  if (denied.has(permission)) return false;
  return allowed.has(permission);
} 