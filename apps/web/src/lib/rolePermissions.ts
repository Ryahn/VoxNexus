/** Tri-state permission JSON helpers (roles + channel overrides). */

export type TriState = 'inherit' | 'allow' | 'deny';

export type PermissionJson = {
  allow?: Record<string, number>;
  deny?: Record<string, number>;
};

export const PERM_BITS: { family: string; bit: number; code: string; label: string }[] = [
  { family: 'text', bit: 1, code: 'text.view', label: 'View channel' },
  { family: 'text', bit: 2, code: 'text.send', label: 'Send messages' },
  { family: 'text', bit: 4096, code: 'text.manage_messages', label: 'Manage messages' },
  { family: 'community', bit: 8, code: 'community.manage_channels', label: 'Manage channels' },
  { family: 'community', bit: 4, code: 'community.manage_roles', label: 'Manage roles' },
  { family: 'community', bit: 16, code: 'community.view_audit', label: 'View audit log' },
  { family: 'community', bit: 1, code: 'community.administrator', label: 'Administrator' },
];

export function emptyPermissions(): PermissionJson {
  return { allow: {}, deny: {} };
}

export function readTri(permissions: PermissionJson, family: string, bit: number): TriState {
  const allow = permissions.allow?.[family] ?? 0;
  const deny = permissions.deny?.[family] ?? 0;
  if (deny & bit) return 'deny';
  if (allow & bit) return 'allow';
  return 'inherit';
}

export function writeTri(
  permissions: PermissionJson,
  family: string,
  bit: number,
  state: TriState,
): PermissionJson {
  const allow = { ...(permissions.allow ?? {}) };
  const deny = { ...(permissions.deny ?? {}) };
  allow[family] = (allow[family] ?? 0) & ~bit;
  deny[family] = (deny[family] ?? 0) & ~bit;
  if (state === 'allow') allow[family] = (allow[family] ?? 0) | bit;
  if (state === 'deny') deny[family] = (deny[family] ?? 0) | bit;
  return { allow, deny };
}

export function isEmptyPermissions(permissions: PermissionJson): boolean {
  const allowEmpty =
    !permissions.allow || Object.values(permissions.allow).every((value) => !value);
  const denyEmpty = !permissions.deny || Object.values(permissions.deny).every((value) => !value);
  return allowEmpty && denyEmpty;
}

export function parsePermissionJson(value: unknown): PermissionJson {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return emptyPermissions();
  const obj = value as PermissionJson;
  return {
    allow: { ...(obj.allow ?? {}) },
    deny: { ...(obj.deny ?? {}) },
  };
}
