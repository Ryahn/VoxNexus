import {
  assignMemberRole,
  type CommunityMemberResponse,
  createRole,
  createRoleGroup,
  deleteRole,
  deleteRoleGroup,
  deleteRoleIcon,
  listCommunityMembers,
  listMemberRoles,
  listRoleGroups,
  listRoles,
  type RoleGroupResponse,
  type RoleResponse,
  removeMemberRole,
  reorderRoles,
  updateRole,
  uploadRoleIcon,
} from '@voxnexus/api-client';
import { ChevronDown, ChevronRight, GripVertical, ImagePlus, Plus, X } from 'lucide-react';
import { type ReactNode, useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { readApiErrorMessage } from '../lib/apiError';
import {
  PERM_BITS,
  parsePermissionJson,
  readTri,
  type TriState,
  writeTri,
} from '../lib/rolePermissions';
import { TriStateToggle } from './TriStateToggle';

type Props = {
  communityId: string;
  canManage: boolean;
};

type DetailTab = 'info' | 'permissions' | 'members' | 'links';
type CardStyle = 'solid' | 'gradient' | 'outline';

type RoleCard = {
  style?: CardStyle;
  blurb?: string;
  show_tag?: boolean;
  glow?: boolean;
  from?: string;
  to?: string;
};

const credentials = { credentials: 'include' as const };

function roleColorCss(color: string): string {
  return color.includes(' ') ? `rgb(${color})` : color;
}

function rgbStringToHex(color: string): string {
  if (color.startsWith('#')) return color.length === 7 ? color : '#36d2cd';
  const parts = color.trim().split(/\s+/).map(Number);
  if (parts.length !== 3 || parts.some((n) => Number.isNaN(n))) return '#36d2cd';
  return `#${parts
    .map((n) =>
      Math.max(0, Math.min(255, Math.round(n)))
        .toString(16)
        .padStart(2, '0'),
    )
    .join('')}`;
}

function hexToRgbString(hex: string): string {
  const h = hex.replace('#', '');
  if (h.length !== 6) return '54 210 205';
  const r = parseInt(h.slice(0, 2), 16);
  const g = parseInt(h.slice(2, 4), 16);
  const b = parseInt(h.slice(4, 6), 16);
  if ([r, g, b].some((n) => Number.isNaN(n))) return '54 210 205';
  return `${r} ${g} ${b}`;
}

function parseCard(value: RoleResponse['role_card']): RoleCard {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return {};
  const obj = value as Record<string, unknown>;
  const style = obj.style;
  return {
    style: style === 'gradient' || style === 'outline' || style === 'solid' ? style : 'solid',
    blurb: typeof obj.blurb === 'string' ? obj.blurb : '',
    show_tag: Boolean(obj.show_tag),
    glow: Boolean(obj.glow),
    from: typeof obj.from === 'string' ? obj.from : '54 210 205',
    to: typeof obj.to === 'string' ? obj.to : '76 159 254',
  };
}

function cardGradientCss(card: RoleCard): string {
  const from = card.from ?? '54 210 205';
  const to = card.to ?? '76 159 254';
  return `linear-gradient(135deg, rgb(${from}), rgb(${to}))`;
}

function roleIconUrl(roleId: string, bust: number): string {
  return `/api/v1/roles/${roleId}/icon?v=${bust}`;
}

function sameGroup(a: RoleResponse, b: RoleResponse): boolean {
  return (a.group_id ?? null) === (b.group_id ?? null);
}

function reorderIdsWithinGroup(
  roles: RoleResponse[],
  dragId: string,
  targetId: string,
): string[] | null {
  const drag = roles.find((r) => r.id === dragId);
  const target = roles.find((r) => r.id === targetId);
  if (!drag || !target || drag.is_everyone || target.is_everyone) return null;
  if (!sameGroup(drag, target)) return null;

  const groupKey = drag.group_id ?? null;
  const inGroup = roles.filter((r) => (r.group_id ?? null) === groupKey && !r.is_everyone);
  const ids = inGroup.map((r) => r.id);
  const from = ids.indexOf(dragId);
  const to = ids.indexOf(targetId);
  if (from < 0 || to < 0) return null;
  ids.splice(from, 1);
  ids.splice(to, 0, dragId);

  const positions = inGroup.map((r) => r.position).sort((a, b) => a - b);
  const positionById = new Map(ids.map((id, index) => [id, positions[index]!]));
  return [...roles]
    .map((role) => ({
      ...role,
      position: positionById.get(role.id) ?? role.position,
    }))
    .sort((a, b) => a.position - b.position || a.created_at.localeCompare(b.created_at))
    .map((role) => role.id);
}

export function CommunityRolesPanel({ communityId, canManage }: Props) {
  const [roles, setRoles] = useState<RoleResponse[]>([]);
  const [groups, setGroups] = useState<RoleGroupResponse[]>([]);
  const [collapsed, setCollapsed] = useState<Record<string, boolean>>({});
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [detailTab, setDetailTab] = useState<DetailTab>('info');
  const [filter, setFilter] = useState('');
  const [newGroupName, setNewGroupName] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const [dragId, setDragId] = useState<string | null>(null);
  const [dropHint, setDropHint] = useState<string | null>(null);
  const [denyWarning, setDenyWarning] = useState<string | null>(null);
  const [memberRows, setMemberRows] = useState<
    { id: string; username: string; nickname: string; has: boolean }[]
  >([]);
  const [memberSearch, setMemberSearch] = useState('');
  const [membersLoading, setMembersLoading] = useState(false);
  const [iconBust, setIconBust] = useState(0);
  const fileRef = useRef<HTMLInputElement>(null);

  const selected = roles.find((r) => r.id === selectedId) ?? null;
  const card = selected ? parseCard(selected.role_card) : null;

  const refresh = useCallback(async () => {
    const [rolesResult, groupsResult] = await Promise.all([
      listRoles({ path: { community_id: communityId } }),
      listRoleGroups({ path: { community_id: communityId } }),
    ]);
    if (rolesResult.error || !rolesResult.data) {
      setError(readApiErrorMessage(rolesResult.error, 'Could not load roles.'));
      return;
    }
    setRoles(rolesResult.data.roles);
    if (groupsResult.data) setGroups(groupsResult.data.groups);
  }, [communityId]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  useEffect(() => {
    if (!selected || detailTab !== 'members') return;
    setMemberSearch('');
    if (selected.is_everyone) {
      setMemberRows([]);
      return;
    }
    void (async () => {
      setMembersLoading(true);
      const items: CommunityMemberResponse[] = [];
      let after: string | undefined;
      for (;;) {
        const page = await listCommunityMembers({
          path: { community_id: communityId },
          query: { limit: 100, after },
        });
        if (!page.data?.items) break;
        items.push(...page.data.items);
        if (!page.data.has_more || page.data.items.length === 0) break;
        after = page.data.items[page.data.items.length - 1]?.account_id;
        if (!after) break;
      }
      const rows = await Promise.all(
        items.map(async (m) => {
          const assigned = await listMemberRoles({
            path: { community_id: communityId, account_id: m.account_id },
          });
          const has = assigned.data?.roles.some((r) => r.id === selected.id) ?? false;
          return {
            id: m.account_id,
            username: m.display_name.trim() || 'Member',
            nickname: m.nickname.trim(),
            has,
          };
        }),
      );
      setMemberRows(rows);
      setMembersLoading(false);
    })();
  }, [selected, detailTab, communityId]);

  const assignedMembers = useMemo(() => memberRows.filter((r) => r.has), [memberRows]);

  const memberSearchHits = useMemo(() => {
    const q = memberSearch.trim().toLowerCase();
    if (!q) return [];
    return memberRows.filter((r) => {
      if (r.has) return false;
      return (
        r.username.toLowerCase().includes(q) ||
        r.nickname.toLowerCase().includes(q) ||
        r.id.toLowerCase().includes(q)
      );
    });
  }, [memberRows, memberSearch]);

  const memberLabel = (row: { username: string; nickname: string }) => row.nickname || row.username;

  const setMemberHas = (accountId: string, has: boolean) => {
    setMemberRows((prev) => prev.map((m) => (m.id === accountId ? { ...m, has } : m)));
  };

  const addMemberToRole = async (accountId: string) => {
    if (!selected || !canManage) return;
    const result = await assignMemberRole({
      path: { community_id: communityId, account_id: accountId },
      body: { role_id: selected.id },
    });
    if (result.error) {
      setError(readApiErrorMessage(result.error, 'Could not add member.'));
      return;
    }
    setMemberHas(accountId, true);
    setMemberSearch('');
  };

  const removeMemberFromRole = async (accountId: string) => {
    if (!selected || !canManage) return;
    const result = await removeMemberRole({
      path: {
        community_id: communityId,
        account_id: accountId,
        role_id: selected.id,
      },
    });
    if (result.error) {
      setError(readApiErrorMessage(result.error, 'Could not remove member.'));
      return;
    }
    setMemberHas(accountId, false);
  };

  const filteredRoles = useMemo(() => {
    const q = filter.trim().toLowerCase();
    if (!q) return roles;
    return roles.filter(
      (r) =>
        r.name.toLowerCase().includes(q) ||
        r.short_tag.toLowerCase().includes(q) ||
        String(r.weight).includes(q),
    );
  }, [roles, filter]);

  const create = async () => {
    if (!canManage) return;
    setPending(true);
    setError(null);
    const result = await createRole({
      path: { community_id: communityId },
      body: { name: 'new role' },
    });
    setPending(false);
    if (result.error || !result.data) {
      setError(readApiErrorMessage(result.error, 'Could not create role.'));
      return;
    }
    await refresh();
    setSelectedId(result.data.id);
    setDetailTab('info');
  };

  const createGroup = async () => {
    const name = newGroupName.trim();
    if (!name || !canManage) return;
    setPending(true);
    const result = await createRoleGroup({
      path: { community_id: communityId },
      body: { name },
    });
    setPending(false);
    if (result.error) {
      setError(readApiErrorMessage(result.error, 'Could not create group.'));
      return;
    }
    setNewGroupName('');
    await refresh();
  };

  const patchSelected = async (body: Parameters<typeof updateRole>[0]['body']) => {
    if (!selected || !canManage) return;
    setPending(true);
    setError(null);
    const result = await updateRole({ path: { role_id: selected.id }, body });
    setPending(false);
    if (result.error || !result.data) {
      setError(readApiErrorMessage(result.error, 'Could not update role.'));
      return;
    }
    setRoles((prev) => prev.map((r) => (r.id === result.data!.id ? result.data! : r)));
  };

  const saveCard = async (next: RoleCard) => {
    const gradient =
      next.style === 'gradient' ? cardGradientCss(next) : (selected?.gradient ?? null);
    await patchSelected({
      role_card: next,
      gradient: next.style === 'gradient' ? gradient : undefined,
      clear_gradient: next.style !== 'gradient' ? true : undefined,
    });
  };

  const onDropReorder = async (targetId: string) => {
    if (!dragId || !canManage) {
      setDragId(null);
      setDropHint(null);
      return;
    }
    const nextIds = reorderIdsWithinGroup(roles, dragId, targetId);
    setDragId(null);
    setDropHint(null);
    if (!nextIds) {
      setError('Roles can only be reordered within the same group.');
      return;
    }
    setPending(true);
    setError(null);
    const result = await reorderRoles({
      path: { community_id: communityId },
      body: { role_ids: nextIds },
    });
    setPending(false);
    if (result.error || !result.data) {
      setError(readApiErrorMessage(result.error, 'Could not reorder roles.'));
      return;
    }
    setRoles(result.data.roles);
  };

  const uploadIcon = async (file: File) => {
    if (!selected || !canManage) return;
    setPending(true);
    setError(null);
    const bytes = new Uint8Array(await file.arrayBuffer());
    const result = await uploadRoleIcon({
      path: { role_id: selected.id },
      body: bytes,
      headers: { 'Content-Type': 'application/octet-stream' },
      ...credentials,
    });
    setPending(false);
    if (result.error || !result.data) {
      setError(readApiErrorMessage(result.error, 'Icon upload failed.'));
      return;
    }
    setRoles((prev) => prev.map((r) => (r.id === result.data!.id ? result.data! : r)));
    setIconBust((v) => v + 1);
  };

  const clearIcon = async () => {
    if (!selected || !canManage) return;
    setPending(true);
    const result = await deleteRoleIcon({
      path: { role_id: selected.id },
      ...credentials,
    });
    setPending(false);
    if (result.error || !result.data) {
      setError(readApiErrorMessage(result.error, 'Could not remove icon.'));
      return;
    }
    setRoles((prev) => prev.map((r) => (r.id === result.data!.id ? result.data! : r)));
    setIconBust((v) => v + 1);
  };

  const setPerm = async (family: string, bit: number, label: string, next: TriState) => {
    if (!selected) return;
    if (next === 'deny') {
      setDenyWarning(
        `Setting Deny on “${label}” may block members of this role from that action, even if a lower-priority role allows it.`,
      );
    } else {
      setDenyWarning(null);
    }
    const permissions = writeTri(parsePermissionJson(selected.permissions), family, bit, next);
    await patchSelected({ permissions });
  };

  const renderRoleRow = (role: RoleResponse) => {
    const dragging = dragId === role.id;
    const dragRole = dragId ? roles.find((r) => r.id === dragId) : null;
    const canDropHere =
      canManage &&
      !role.is_everyone &&
      dragRole &&
      !dragRole.is_everyone &&
      sameGroup(dragRole, role);
    return (
      <button
        key={role.id}
        type="button"
        draggable={canManage && !role.is_everyone}
        onDragStart={() => {
          setDragId(role.id);
          setDropHint(null);
          setError(null);
        }}
        onDragEnd={() => {
          setDragId(null);
          setDropHint(null);
        }}
        onDragOver={(e) => {
          if (!dragRole) return;
          e.preventDefault();
          if (!sameGroup(dragRole, role) || role.is_everyone) {
            setDropHint('Stay within the same group');
            e.dataTransfer.dropEffect = 'none';
          } else {
            setDropHint(null);
            e.dataTransfer.dropEffect = 'move';
          }
        }}
        onDrop={() => void onDropReorder(role.id)}
        onClick={() => {
          setSelectedId(role.id);
          setDetailTab('info');
          setDenyWarning(null);
        }}
        className={`flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-[13px] transition-colors ${
          selectedId === role.id
            ? 'bg-surface-active text-ink'
            : 'text-ink-2 hover:bg-surface-hover/70 hover:text-ink'
        } ${dragging ? 'opacity-50' : ''} ${
          canDropHere && dragId && dragId !== role.id ? 'ring-1 ring-accent/50' : ''
        }`}
      >
        {canManage && !role.is_everyone ? (
          <GripVertical
            size={14}
            className="shrink-0 cursor-grab text-ink-3 active:cursor-grabbing"
          />
        ) : (
          <span className="w-3.5" />
        )}
        {role.icon_object_key ? (
          <img
            src={roleIconUrl(role.id, iconBust)}
            alt=""
            className="h-4 w-4 shrink-0 rounded-full object-cover"
          />
        ) : (
          <span
            className="h-2.5 w-2.5 shrink-0 rounded-full"
            style={{ background: roleColorCss(role.color) }}
          />
        )}
        <span className="min-w-0 flex-1 truncate font-medium">
          {role.icon_emoji ? `${role.icon_emoji} ` : ''}
          {role.name}
        </span>
        <RoleWeightBadge weight={role.weight} />
      </button>
    );
  };

  return (
    <div className="flex h-full min-h-0">
      <div className="flex w-[280px] shrink-0 flex-col border-r border-line/70 bg-panel/40">
        <div className="border-b border-line/60 px-3 py-3">
          <div className="mb-2 flex items-center justify-between">
            <h2 className="text-sm font-semibold text-ink">Roles</h2>
            {canManage ? (
              <button
                type="button"
                disabled={pending}
                onClick={() => void create()}
                className="grid h-7 w-7 place-items-center rounded-md text-ink-2 hover:bg-surface-hover hover:text-ink"
                title="Create role"
              >
                <Plus size={16} />
              </button>
            ) : null}
          </div>
          <input
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            placeholder="Filter roles or groups"
            className="w-full rounded-md border border-line-2/70 bg-surface px-2 py-1.5 text-[12px] text-ink outline-none focus:border-accent/50"
          />
          {dropHint ? <p className="mt-1 text-2xs text-amber-300">{dropHint}</p> : null}
        </div>
        <div className="min-h-0 flex-1 overflow-y-auto px-2 py-2">
          {groups.map((group) => {
            const open = !collapsed[group.id];
            const inGroup = filteredRoles.filter((r) => r.group_id === group.id);
            return (
              <div key={group.id} className="mb-2">
                <button
                  type="button"
                  onClick={() => setCollapsed((c) => ({ ...c, [group.id]: !c[group.id] }))}
                  className="flex w-full items-center gap-1 px-1 py-1 text-left text-2xs font-semibold uppercase tracking-wide text-ink-3"
                >
                  {open ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
                  {group.name}
                  <span className="ml-auto font-normal normal-case">{inGroup.length}</span>
                </button>
                {open ? inGroup.map(renderRoleRow) : null}
              </div>
            );
          })}
          <div className="mb-1 px-1 py-1 text-2xs font-semibold uppercase tracking-wide text-ink-3">
            Ungrouped
          </div>
          {filteredRoles.filter((r) => !r.group_id).map(renderRoleRow)}
        </div>
        {canManage ? (
          <div className="border-t border-line/60 p-2">
            <div className="flex gap-1">
              <input
                value={newGroupName}
                onChange={(e) => setNewGroupName(e.target.value)}
                placeholder="New group (e.g. Vanity)"
                className="min-w-0 flex-1 rounded-md border border-line-2/70 bg-surface px-2 py-1.5 text-[12px] text-ink"
              />
              <button
                type="button"
                disabled={pending}
                onClick={() => void createGroup()}
                className="rounded-md bg-accent px-2 py-1 text-[12px] font-medium text-app"
              >
                Add
              </button>
            </div>
          </div>
        ) : null}
      </div>

      <div className="min-w-0 flex-1 overflow-y-auto px-6 py-5">
        {error ? <p className="mb-3 text-sm text-[rgb(var(--danger))]">{error}</p> : null}
        {!selected || !card ? (
          <p className="text-sm text-ink-3">Select a role or create one to edit it.</p>
        ) : (
          <>
            <RoleCardPreview role={selected} card={card} iconBust={iconBust} />
            <div className="mb-5 flex gap-1 border-b border-line/60">
              {(['info', 'permissions', 'members', 'links'] as DetailTab[]).map((tab) => (
                <button
                  key={tab}
                  type="button"
                  onClick={() => setDetailTab(tab)}
                  className={`px-3 py-2 text-[13px] font-medium capitalize ${
                    detailTab === tab
                      ? 'border-b-2 border-accent text-ink'
                      : 'text-ink-3 hover:text-ink-2'
                  }`}
                >
                  {tab}
                </button>
              ))}
            </div>

            {detailTab === 'info' && (
              <div className="max-w-lg space-y-4">
                <Field label="Display name">
                  <input
                    key={`${selected.id}-name`}
                    defaultValue={selected.name}
                    disabled={!canManage || selected.is_everyone}
                    onBlur={(e) => {
                      const name = e.target.value.trim();
                      if (name && name !== selected.name) void patchSelected({ name });
                    }}
                    className="field"
                  />
                </Field>
                <Field label="Short tag">
                  <input
                    key={`${selected.id}-tag`}
                    defaultValue={selected.short_tag}
                    disabled={!canManage}
                    maxLength={16}
                    onBlur={(e) => {
                      const short_tag = e.target.value.trim();
                      if (short_tag !== selected.short_tag) void patchSelected({ short_tag });
                    }}
                    className="field"
                  />
                </Field>
                <Field label="Weight (1–1000, unique; lower = higher priority)">
                  <input
                    key={`${selected.id}-weight`}
                    type="number"
                    min={1}
                    max={1000}
                    defaultValue={selected.weight}
                    disabled={!canManage || selected.is_everyone}
                    onBlur={(e) => {
                      const weight = Number(e.target.value);
                      if (weight !== selected.weight) void patchSelected({ weight });
                    }}
                    className="field"
                  />
                </Field>
                <Field label="Color">
                  <div className="mt-1 flex items-center gap-3">
                    <input
                      key={`${selected.id}-color`}
                      type="color"
                      defaultValue={rgbStringToHex(selected.color)}
                      disabled={!canManage}
                      onChange={(e) => {
                        const color = hexToRgbString(e.target.value);
                        if (color !== selected.color) void patchSelected({ color });
                      }}
                      className="role-color-picker"
                      aria-label="Role color"
                    />
                    <span className="font-mono text-sm text-ink-2">{selected.color}</span>
                    <span
                      className="ml-auto h-8 w-8 shrink-0 rounded-lg border border-line/60"
                      style={{ background: roleColorCss(selected.color) }}
                      aria-hidden
                    />
                  </div>
                </Field>

                <div className="rounded-xl border border-line/60 p-3">
                  <div className="mb-2 text-xs font-medium uppercase tracking-wide text-ink-3">
                    Icon
                  </div>
                  <div className="flex items-center gap-3">
                    <div className="grid h-14 w-14 place-items-center overflow-hidden rounded-xl border border-line/60 bg-surface">
                      {selected.icon_object_key ? (
                        <img
                          src={roleIconUrl(selected.id, iconBust)}
                          alt=""
                          className="h-full w-full object-cover"
                        />
                      ) : selected.icon_emoji ? (
                        <span className="text-2xl">{selected.icon_emoji}</span>
                      ) : (
                        <span
                          className="h-6 w-6 rounded-full"
                          style={{ background: roleColorCss(selected.color) }}
                        />
                      )}
                    </div>
                    <div className="min-w-0 flex-1 space-y-2">
                      <Field label="Emoji">
                        <input
                          key={`${selected.id}-emoji`}
                          defaultValue={selected.icon_emoji ?? ''}
                          disabled={!canManage}
                          placeholder="🛡️"
                          onBlur={(e) => {
                            const trimmed = e.target.value.trim();
                            if (!trimmed) void patchSelected({ clear_icon_emoji: true });
                            else if (trimmed !== (selected.icon_emoji ?? '')) {
                              void patchSelected({ icon_emoji: trimmed });
                            }
                          }}
                          className="field"
                        />
                      </Field>
                      {canManage ? (
                        <div className="flex flex-wrap gap-2">
                          <input
                            ref={fileRef}
                            type="file"
                            accept="image/png,image/jpeg,image/gif,image/webp"
                            className="hidden"
                            onChange={(e) => {
                              const file = e.target.files?.[0];
                              e.target.value = '';
                              if (file) void uploadIcon(file);
                            }}
                          />
                          <button
                            type="button"
                            disabled={pending}
                            onClick={() => fileRef.current?.click()}
                            className="inline-flex items-center gap-1.5 rounded-lg border border-line px-2.5 py-1.5 text-[12px] text-ink-2 hover:bg-surface-hover"
                          >
                            <ImagePlus size={14} /> Upload image
                          </button>
                          {selected.icon_object_key ? (
                            <button
                              type="button"
                              disabled={pending}
                              onClick={() => void clearIcon()}
                              className="inline-flex items-center gap-1 rounded-lg border border-line px-2.5 py-1.5 text-[12px] text-ink-3 hover:bg-surface-hover"
                            >
                              <X size={14} /> Remove image
                            </button>
                          ) : null}
                        </div>
                      ) : null}
                    </div>
                  </div>
                </div>

                <div className="rounded-xl border border-line/60 p-3 space-y-3">
                  <div className="text-xs font-medium uppercase tracking-wide text-ink-3">
                    Role card
                  </div>
                  <Field label="Style">
                    <select
                      value={card.style ?? 'solid'}
                      disabled={!canManage}
                      onChange={(e) =>
                        void saveCard({ ...card, style: e.target.value as CardStyle })
                      }
                      className="field"
                    >
                      <option value="solid">Solid</option>
                      <option value="gradient">Gradient</option>
                      <option value="outline">Outline</option>
                    </select>
                  </Field>
                  {(card.style ?? 'solid') === 'gradient' ? (
                    <div className="grid grid-cols-2 gap-2">
                      <Field label="From (r g b)">
                        <input
                          key={`${selected.id}-from`}
                          defaultValue={card.from}
                          disabled={!canManage}
                          onBlur={(e) => {
                            const from = e.target.value.trim();
                            if (from && from !== card.from) void saveCard({ ...card, from });
                          }}
                          className="field"
                        />
                      </Field>
                      <Field label="To (r g b)">
                        <input
                          key={`${selected.id}-to`}
                          defaultValue={card.to}
                          disabled={!canManage}
                          onBlur={(e) => {
                            const to = e.target.value.trim();
                            if (to && to !== card.to) void saveCard({ ...card, to });
                          }}
                          className="field"
                        />
                      </Field>
                    </div>
                  ) : null}
                  <Field label="Blurb">
                    <textarea
                      key={`${selected.id}-blurb`}
                      defaultValue={card.blurb}
                      disabled={!canManage}
                      maxLength={200}
                      rows={2}
                      onBlur={(e) => {
                        const blurb = e.target.value.trim();
                        if (blurb !== (card.blurb ?? '')) void saveCard({ ...card, blurb });
                      }}
                      className="field resize-y"
                    />
                  </Field>
                  <label className="flex items-center gap-2 text-sm text-ink">
                    <input
                      type="checkbox"
                      checked={Boolean(card.show_tag)}
                      disabled={!canManage}
                      onChange={(e) => void saveCard({ ...card, show_tag: e.target.checked })}
                    />
                    Show short tag on card
                  </label>
                  <label className="flex items-center gap-2 text-sm text-ink">
                    <input
                      type="checkbox"
                      checked={Boolean(card.glow)}
                      disabled={!canManage}
                      onChange={(e) => void saveCard({ ...card, glow: e.target.checked })}
                    />
                    Soft glow
                  </label>
                </div>

                <Field label="Group">
                  <select
                    value={selected.group_id ?? ''}
                    disabled={!canManage || selected.is_everyone}
                    onChange={(e) => {
                      const value = e.target.value;
                      if (value) void patchSelected({ group_id: value, clear_group: false });
                      else void patchSelected({ clear_group: true });
                    }}
                    className="field"
                  >
                    <option value="">Ungrouped</option>
                    {groups.map((g) => (
                      <option key={g.id} value={g.id}>
                        {g.name}
                      </option>
                    ))}
                  </select>
                </Field>
                <label className="flex items-center gap-2 text-sm text-ink">
                  <input
                    type="checkbox"
                    checked={selected.hoist}
                    disabled={!canManage}
                    onChange={(e) => void patchSelected({ hoist: e.target.checked })}
                  />
                  Hoist (display separately)
                </label>
                <label className="flex items-center gap-2 text-sm text-ink">
                  <input
                    type="checkbox"
                    checked={selected.mentionable}
                    disabled={!canManage}
                    onChange={(e) => void patchSelected({ mentionable: e.target.checked })}
                  />
                  Mentionable
                </label>
                {canManage && !selected.is_everyone ? (
                  <button
                    type="button"
                    disabled={pending}
                    onClick={() =>
                      void (async () => {
                        const result = await deleteRole({ path: { role_id: selected.id } });
                        if (result.error) {
                          setError(readApiErrorMessage(result.error, 'Could not delete role.'));
                          return;
                        }
                        setSelectedId(null);
                        await refresh();
                      })()
                    }
                    className="mt-2 rounded-lg border border-[rgb(var(--danger))]/40 px-3 py-2 text-sm text-[rgb(var(--danger))]"
                  >
                    Delete role
                  </button>
                ) : null}
                {canManage && selected.group_id ? (
                  <button
                    type="button"
                    className="ml-2 mt-2 rounded-lg border border-line px-3 py-2 text-sm text-ink-2"
                    onClick={() =>
                      void (async () => {
                        const gid = selected.group_id!;
                        const result = await deleteRoleGroup({ path: { group_id: gid } });
                        if (result.error) {
                          setError(readApiErrorMessage(result.error, 'Could not delete group.'));
                          return;
                        }
                        await refresh();
                      })()
                    }
                  >
                    Delete this group
                  </button>
                ) : null}
              </div>
            )}

            {detailTab === 'permissions' && (
              <div className="max-w-lg space-y-3">
                <p className="text-sm text-ink-3">
                  Allow / Deny / Inherit. Lower weight wins on conflict. Channel overrides still
                  apply unless Administrator is granted.
                </p>
                {denyWarning ? (
                  <p className="rounded-md border border-amber-500/40 bg-amber-500/10 px-3 py-2 text-sm text-amber-200">
                    {denyWarning}
                  </p>
                ) : null}
                {PERM_BITS.map((perm) => {
                  const value = readTri(
                    parsePermissionJson(selected.permissions),
                    perm.family,
                    perm.bit,
                  );
                  return (
                    <div
                      key={perm.code}
                      className="flex items-center justify-between gap-3 rounded-lg border border-line/50 px-3 py-2"
                    >
                      <div>
                        <div className="text-sm font-medium text-ink">{perm.label}</div>
                        <div className="font-mono text-2xs text-ink-3">{perm.code}</div>
                      </div>
                      <TriStateToggle
                        value={value}
                        disabled={!canManage}
                        onChange={(next) => void setPerm(perm.family, perm.bit, perm.label, next)}
                      />
                    </div>
                  );
                })}
              </div>
            )}

            {detailTab === 'members' && (
              <div className="max-w-lg space-y-6">
                {selected.is_everyone ? (
                  <p className="text-sm text-ink-3">@everyone applies to all community members.</p>
                ) : membersLoading ? (
                  <p className="text-sm text-ink-3">Loading members…</p>
                ) : (
                  <>
                    <section className="space-y-3">
                      <div>
                        <h3 className="text-sm font-semibold text-ink">Add members</h3>
                        <p className="mt-0.5 text-[12px] text-ink-3">
                          Search by username or account ID, then add them to this role.
                        </p>
                      </div>
                      <input
                        type="search"
                        value={memberSearch}
                        onChange={(e) => setMemberSearch(e.target.value)}
                        placeholder="Search by username or ID…"
                        className="field mt-0"
                        autoComplete="off"
                        disabled={!canManage}
                      />
                      {memberSearch.trim() ? (
                        <ul className="max-h-48 space-y-1 overflow-y-auto rounded-lg border border-line/50 p-1">
                          {memberSearchHits.length === 0 ? (
                            <li className="px-3 py-2 text-[13px] text-ink-4">
                              No matching members.
                            </li>
                          ) : (
                            memberSearchHits.map((row) => (
                              <li
                                key={row.id}
                                className="flex items-center gap-2 rounded-md px-2 py-1.5 hover:bg-surface-hover/50"
                              >
                                <div className="min-w-0 flex-1">
                                  <p className="truncate text-[13px] text-ink">
                                    {memberLabel(row)}
                                  </p>
                                  <p className="truncate font-mono text-[11px] text-ink-4">
                                    {row.id}
                                  </p>
                                </div>
                                {canManage ? (
                                  <button
                                    type="button"
                                    onClick={() => void addMemberToRole(row.id)}
                                    className="shrink-0 rounded-md bg-accent px-2.5 py-1 text-[12px] font-medium text-app"
                                  >
                                    Add
                                  </button>
                                ) : null}
                              </li>
                            ))
                          )}
                        </ul>
                      ) : (
                        <p className="text-[12px] text-ink-4">
                          Type to find members without this role.
                        </p>
                      )}
                    </section>

                    <section className="space-y-3">
                      <div className="flex items-baseline justify-between gap-2">
                        <h3 className="text-sm font-semibold text-ink">Members with this role</h3>
                        <span className="text-[12px] text-ink-4">{assignedMembers.length}</span>
                      </div>
                      <ul className="max-h-72 space-y-1 overflow-y-auto rounded-lg border border-line/50 p-1">
                        {assignedMembers.length === 0 ? (
                          <li className="px-3 py-2 text-[13px] text-ink-4">
                            No members have this role yet.
                          </li>
                        ) : (
                          assignedMembers.map((row) => (
                            <li
                              key={row.id}
                              className="flex items-center gap-2 rounded-md px-2 py-1.5 hover:bg-surface-hover/50"
                            >
                              <div className="min-w-0 flex-1">
                                <p className="truncate text-[13px] text-ink">{memberLabel(row)}</p>
                                <p className="truncate font-mono text-[11px] text-ink-4">
                                  {row.id}
                                </p>
                              </div>
                              {canManage ? (
                                <button
                                  type="button"
                                  onClick={() => void removeMemberFromRole(row.id)}
                                  className="shrink-0 rounded-md px-2 py-1 text-[12px] text-dnd hover:bg-dnd/10"
                                  aria-label={`Remove ${memberLabel(row)}`}
                                >
                                  Remove
                                </button>
                              ) : null}
                            </li>
                          ))
                        )}
                      </ul>
                    </section>
                  </>
                )}
              </div>
            )}

            {detailTab === 'links' && (
              <div className="max-w-lg space-y-3">
                <p className="text-sm text-ink-3">
                  Link OIDC claims and identity-provider groups to this role. Full linked-role
                  management arrives in F090; this tab will host those rules.
                </p>
                <div className="rounded-lg border border-dashed border-line/60 px-4 py-6 text-center text-[13px] text-ink-4">
                  Linked roles — coming with SSO claim rules (F090).
                </div>
              </div>
            )}
          </>
        )}
      </div>
      <style>{`
        .field {
          margin-top: 0.25rem;
          width: 100%;
          border-radius: 0.5rem;
          border: 1px solid color-mix(in srgb, var(--line-2, #444) 80%, transparent);
          background: var(--surface, #1a1a1a);
          padding: 0.5rem 0.75rem;
          font-size: 0.875rem;
          color: inherit;
          outline: none;
        }
        .role-color-picker {
          height: 2.5rem;
          width: 2.5rem;
          flex-shrink: 0;
          cursor: pointer;
          border-radius: 0.5rem;
          border: 1px solid color-mix(in srgb, var(--line-2, #444) 80%, transparent);
          padding: 0.125rem;
          background: var(--surface, #1a1a1a);
        }
        .role-color-picker:disabled {
          cursor: not-allowed;
          opacity: 0.6;
        }
        .role-color-picker::-webkit-color-swatch-wrapper {
          padding: 0;
        }
        .role-color-picker::-webkit-color-swatch {
          border: none;
          border-radius: 0.375rem;
        }
        .role-color-picker::-moz-color-swatch {
          border: none;
          border-radius: 0.375rem;
        }
      `}</style>
    </div>
  );
}

function RoleCardPreview({
  role,
  card,
  iconBust,
}: {
  role: RoleResponse;
  card: RoleCard;
  iconBust: number;
}) {
  const style = card.style ?? 'solid';
  const background =
    style === 'gradient'
      ? cardGradientCss(card)
      : style === 'outline'
        ? 'transparent'
        : roleColorCss(role.color);
  return (
    <div
      className={`mb-5 overflow-hidden rounded-2xl border p-4 ${
        style === 'outline' ? 'border-2' : 'border-line/50'
      }`}
      style={{
        background,
        borderColor: style === 'outline' ? roleColorCss(role.color) : undefined,
        boxShadow: card.glow
          ? `0 0 24px color-mix(in srgb, ${roleColorCss(role.color)} 45%, transparent)`
          : undefined,
      }}
    >
      <div className="flex items-center gap-3">
        {role.icon_object_key ? (
          <img
            src={roleIconUrl(role.id, iconBust)}
            alt=""
            className="h-10 w-10 rounded-xl object-cover"
          />
        ) : (
          <div className="grid h-10 w-10 place-items-center rounded-xl bg-black/20 text-lg">
            {role.icon_emoji ?? '◆'}
          </div>
        )}
        <div className="min-w-0">
          <div className="flex items-center gap-2">
            <h1 className="truncate text-lg font-semibold text-white drop-shadow">{role.name}</h1>
            {card.show_tag && role.short_tag ? (
              <span className="rounded bg-black/25 px-1.5 py-0.5 font-mono text-2xs text-white/90">
                {role.short_tag}
              </span>
            ) : null}
            <RoleWeightBadge weight={role.weight} onCard />
          </div>
        </div>
      </div>
      {card.blurb ? <p className="mt-3 text-sm text-white/90">{card.blurb}</p> : null}
    </div>
  );
}

function RoleWeightBadge({ weight, onCard = false }: { weight: number; onCard?: boolean }) {
  return (
    <span
      className={`shrink-0 rounded-md px-1.5 py-0.5 font-mono text-3xs font-semibold tabular-nums ${
        onCard ? 'bg-black/35 text-white/90' : 'border border-line/60 bg-surface-active text-ink'
      }`}
      title={`Weight ${weight}`}
    >
      w{weight}
    </span>
  );
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="block text-xs font-medium uppercase tracking-wide text-ink-3">
      <span className="block">{label}</span>
      {children}
    </div>
  );
}
