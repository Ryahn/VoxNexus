import Role, { IRole } from '../models/Role';
import { PERMISSIONS } from '../types/permissions';

export async function seedDefaultRoles(serverId: string): Promise<IRole[]> {
  // @everyone role
  const everyoneRole = await Role.create({
    serverId,
    name: '@everyone',
    priority: 0,
    permissions: [PERMISSIONS.VIEW_CHANNEL, PERMISSIONS.SEND_MESSAGES],
    isDefault: true,
  });
  return [everyoneRole];
} 