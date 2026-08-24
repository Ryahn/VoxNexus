import {
  deletePermissionOverride,
  type ExplainPermissionResponse,
  explainPermission,
  listCategoryPermissionOverrides,
  listChannelPermissionOverrides,
  listCommunityMembers,
  listRoles,
  type PermissionOverrideResponse,
  type RoleResponse,
  upsertCategoryRolePermissionOverride,
  upsertChannelRolePermissionOverride,
} from '@voxnexus/api-client';
import { Shield, X } from 'lucide-react';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { useAuth } from '../auth';
import { readApiErrorMessage } from '../lib/apiError';
import {
  emptyPermissions,
  isEmptyPermissions,
  PERM_BITS,
  type PermissionJson,
  parsePermissionJson,
  readTri,
  writeTri,
} from '../lib/rolePermissions';
import { type PermissionOverrideTarget, useUI } from '../store';
import { TriStateToggle } from './TriStateToggle';
import { Portal } from './ui/Portal';

function roleColorCss(color: string): string {
  return color.includes(' ') ? `rgb(${color})` : color;
}

function sortRoles(roles: RoleResponse[]): RoleResponse[] {
  return [...roles].sort((a, b) => {
    if (a.is_everyone !== b.is_everyone) return a.is_everyone ? -1 : 1;
    return a.weight - b.weight || a.name.localeCompare(b.name);
  });
}

function scopedOverrideForRole(
  overrides: PermissionOverrideResponse[],
  target: PermissionOverrideTarget,
  roleId: string,
): PermissionOverrideResponse | undefined {
  if (target.scope === 'channel') {
    return overrides.find((row) => row.channel_id === target.channelId && row.role_id === roleId);
  }
  return overrides.find(
    (row) => row.category_id === target.categoryId && !row.channel_id && row.role_id === roleId,
  );
}

function inheritedCategoryOverride(
  overrides: PermissionOverrideResponse[],
  target: PermissionOverrideTarget,
  roleId: string,
): PermissionOverrideResponse | undefined {
  if (target.scope !== 'channel' || !target.categoryId) return undefined;
  return overrides.find(
    (row) => row.category_id === target.categoryId && !row.channel_id && row.role_id === roleId,
  );
}

function summarizeOverride(permissions: PermissionJson): string {
  const labels = PERM_BITS.filter(
    (perm) => readTri(permissions, perm.family, perm.bit) !== 'inherit',
  ).map((perm) => {
    const state = readTri(permissions, perm.family, perm.bit);
    return `${perm.label} (${state})`;
  });
  return labels.length > 0 ? labels.join(', ') : 'No overrides set for this role.';
}

export function ChannelPermissionsModal() {
  const { session } = useAuth();
  const target = useUI((s) => s.permissionOverrides);
  const close = useUI((s) => s.closePermissionOverrides);
  const [mode, setMode] = useState<'overrides' | 'explain'>('overrides');
  const [roles, setRoles] = useState<RoleResponse[]>([]);
  const [overrides, setOverrides] = useState<PermissionOverrideResponse[]>([]);
  const [selectedRoleId, setSelectedRoleId] = useState<string | null>(null);
  const [draft, setDraft] = useState<PermissionJson>(emptyPermissions());
  const [overrideId, setOverrideId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const [explainAccountId, setExplainAccountId] = useState('');
  const [explainPermissionCode, setExplainPermissionCode] = useState('text.view');
  const [explainResult, setExplainResult] = useState<ExplainPermissionResponse | null>(null);
  const [memberOptions, setMemberOptions] = useState<{ id: string; label: string }[]>([]);

  const open = target !== null;
  const selectedRole = roles.find((role) => role.id === selectedRoleId) ?? null;
  const isChannel = target?.scope === 'channel';

  const refresh = useCallback(async () => {
    if (!target) return;
    const rolesResult = await listRoles({ path: { community_id: target.communityId } });
    const overridesResult =
      target.scope === 'channel'
        ? await listChannelPermissionOverrides({ path: { channel_id: target.channelId } })
        : await listCategoryPermissionOverrides({ path: { category_id: target.categoryId } });
    if (rolesResult.error || !rolesResult.data) {
      setError(readApiErrorMessage(rolesResult.error, 'Could not load roles.'));
      return;
    }
    if (overridesResult.error || !overridesResult.data) {
      setError(readApiErrorMessage(overridesResult.error, 'Could not load overrides.'));
      return;
    }
    setError(null);
    const sorted = sortRoles(rolesResult.data.roles);
    setRoles(sorted);
    setOverrides(overridesResult.data.overrides);
    setSelectedRoleId((prev) => {
      if (prev && sorted.some((role) => role.id === prev)) return prev;
      return sorted[0]?.id ?? null;
    });
  }, [target]);

  useEffect(() => {
    if (!open) return;
    setError(null);
    setPending(false);
    setMode('overrides');
    setExplainResult(null);
    setExplainPermissionCode('text.view');
    void refresh();
    void (async () => {
      if (!target) return;
      const page = await listCommunityMembers({
        path: { community_id: target.communityId },
        query: { limit: 100 },
      });
      if (page.data?.items) {
        setMemberOptions(
          page.data.items.map((m) => ({
            id: m.account_id,
            label: m.nickname.trim() || m.display_name.trim() || m.account_id,
          })),
        );
      }
    })();
  }, [open, refresh, target]);

  useEffect(() => {
    if (!open || !target || !selectedRoleId) {
      setDraft(emptyPermissions());
      setOverrideId(null);
      return;
    }
    const row = scopedOverrideForRole(overrides, target, selectedRoleId);
    setDraft(row ? parsePermissionJson(row.permissions) : emptyPermissions());
    setOverrideId(row?.id ?? null);
  }, [open, overrides, selectedRoleId, target]);

  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') close();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [close, open]);

  const inheritedSummary = useMemo(() => {
    if (!target || !selectedRoleId || target.scope !== 'channel') return null;
    const row = inheritedCategoryOverride(overrides, target, selectedRoleId);
    if (!row) return null;
    return summarizeOverride(parsePermissionJson(row.permissions));
  }, [overrides, selectedRoleId, target]);

  if (!open || !target) return null;

  const savePermissions = async (next: PermissionJson) => {
    setPending(true);
    setError(null);
    if (isEmptyPermissions(next)) {
      if (overrideId) {
        const result = await deletePermissionOverride({
          path: { community_id: target.communityId, override_id: overrideId },
        });
        setPending(false);
        if (result.error) {
          setError(readApiErrorMessage(result.error, 'Could not clear override.'));
          return;
        }
      } else {
        setPending(false);
      }
      setDraft(emptyPermissions());
      setOverrideId(null);
      await refresh();
      return;
    }
    const result =
      target.scope === 'channel'
        ? await upsertChannelRolePermissionOverride({
            path: { channel_id: target.channelId, role_id: selectedRoleId! },
            body: { permissions: next },
          })
        : await upsertCategoryRolePermissionOverride({
            path: { category_id: target.categoryId, role_id: selectedRoleId! },
            body: { permissions: next },
          });
    setPending(false);
    if (result.error || !result.data) {
      setError(readApiErrorMessage(result.error, 'Could not save override.'));
      return;
    }
    setDraft(parsePermissionJson(result.data.permissions));
    setOverrideId(result.data.id);
    await refresh();
  };

  const setPerm = async (family: string, bit: number, next: Parameters<typeof writeTri>[3]) => {
    const updated = writeTri(draft, family, bit, next);
    setDraft(updated);
    await savePermissions(updated);
  };

  const runExplain = async () => {
    if (!target) return;
    const accountId = explainAccountId.trim() || session.account.id;
    setPending(true);
    setError(null);
    const result = await explainPermission({
      body: {
        community_id: target.communityId,
        account_id: accountId,
        permission: explainPermissionCode,
        channel_id: target.scope === 'channel' ? target.channelId : null,
      },
    });
    setPending(false);
    if (result.error || !result.data) {
      setExplainResult(null);
      setError(readApiErrorMessage(result.error, 'Could not explain permission.'));
      return;
    }
    setExplainResult(result.data);
  };

  const title = isChannel ? 'Channel permissions' : 'Category permissions';
  const subtitle = isChannel ? `#${target.name}` : target.name;

  return (
    <Portal>
      <div className="fixed inset-0 z-[85] grid place-items-center p-4">
        <button
          type="button"
          aria-label="Close"
          className="absolute inset-0 bg-black/55"
          onClick={close}
        />
        <div
          role="dialog"
          aria-modal="true"
          aria-labelledby="permission-overrides-title"
          className="relative flex h-[min(640px,90vh)] w-full max-w-3xl flex-col overflow-hidden rounded-2xl border border-line/80 bg-panel shadow-xl"
        >
          <header className="flex items-center gap-3 border-b border-line/70 px-5 py-4">
            <span className="grid h-9 w-9 place-items-center rounded-lg bg-accent/15 text-accent">
              <Shield size={18} strokeWidth={1.9} />
            </span>
            <div className="min-w-0 flex-1">
              <h2
                id="permission-overrides-title"
                className="truncate text-lg font-semibold text-ink"
              >
                {title}
              </h2>
              <p className="truncate text-sm text-ink-3">{subtitle}</p>
            </div>
            <button
              type="button"
              aria-label="Close"
              onClick={close}
              className="grid h-8 w-8 place-items-center rounded-md text-ink-3 hover:bg-surface-hover hover:text-ink"
            >
              <X size={18} />
            </button>
          </header>

          {error ? (
            <p className="border-b border-dnd/30 bg-dnd/10 px-5 py-2 text-sm text-dnd">{error}</p>
          ) : null}

          <div className="flex gap-1 border-b border-line/60 px-5 py-2">
            {(['overrides', 'explain'] as const).map((tab) => (
              <button
                key={tab}
                type="button"
                onClick={() => setMode(tab)}
                className={`rounded-md px-3 py-1.5 text-[13px] font-medium transition-colors ${
                  mode === tab
                    ? 'bg-surface-active text-ink'
                    : 'text-ink-3 hover:bg-surface-hover hover:text-ink-2'
                }`}
              >
                {tab === 'overrides' ? 'Overrides' : 'Explain access'}
              </button>
            ))}
          </div>

          {mode === 'explain' ? (
            <div className="min-h-0 flex-1 overflow-y-auto p-5">
              <div className="max-w-lg space-y-4">
                <p className="text-sm text-ink-3">
                  See why a member is allowed or denied a permission on this{' '}
                  {isChannel ? 'channel' : 'category'}.
                </p>
                <label className="block text-xs font-medium uppercase tracking-wide text-ink-3">
                  Member
                  <select
                    value={explainAccountId}
                    onChange={(e) => setExplainAccountId(e.target.value)}
                    className="mt-1 w-full rounded-lg border border-line-2/80 bg-surface px-3 py-2 text-sm text-ink outline-none focus:border-accent/50"
                  >
                    <option value="">Your account</option>
                    {memberOptions.map((m) => (
                      <option key={m.id} value={m.id}>
                        {m.label}
                      </option>
                    ))}
                  </select>
                </label>
                <label className="block text-xs font-medium uppercase tracking-wide text-ink-3">
                  Permission
                  <select
                    value={explainPermissionCode}
                    onChange={(e) => setExplainPermissionCode(e.target.value)}
                    className="mt-1 w-full rounded-lg border border-line-2/80 bg-surface px-3 py-2 text-sm text-ink outline-none focus:border-accent/50"
                  >
                    {PERM_BITS.map((perm) => (
                      <option key={perm.code} value={perm.code}>
                        {perm.label} ({perm.code})
                      </option>
                    ))}
                  </select>
                </label>
                <button
                  type="button"
                  disabled={pending}
                  onClick={() => void runExplain()}
                  className="rounded-lg bg-accent px-4 py-2 text-sm font-medium text-app disabled:opacity-60"
                >
                  {pending ? 'Explaining…' : 'Explain'}
                </button>
                {explainResult ? (
                  <div className="space-y-3 rounded-lg border border-line/50 p-3">
                    <p className="text-sm font-semibold text-ink">
                      Result:{' '}
                      <span className={explainResult.allowed ? 'text-emerald-400' : 'text-red-400'}>
                        {explainResult.allowed ? 'Allowed' : 'Denied'}
                      </span>
                    </p>
                    <ol className="space-y-2">
                      {explainResult.steps.map((step, index) => (
                        <li
                          key={`${step.stage}-${index}`}
                          className="rounded-md border border-line/40 px-3 py-2 text-[13px]"
                        >
                          <div className="flex items-center justify-between gap-2">
                            <span className="font-mono text-2xs uppercase text-ink-4">
                              {step.stage}
                            </span>
                            <span className="font-mono text-2xs text-accent">{step.outcome}</span>
                          </div>
                          <p className="mt-1 text-ink-2">{step.detail}</p>
                        </li>
                      ))}
                    </ol>
                  </div>
                ) : null}
              </div>
            </div>
          ) : (
            <div className="grid min-h-0 flex-1 grid-cols-[220px_1fr]">
              <aside className="overflow-y-auto border-r border-line/60 bg-surface/40 p-2">
                <p className="px-2 pb-2 text-[11px] font-semibold uppercase tracking-wide text-ink-4">
                  Roles
                </p>
                <ul className="space-y-0.5">
                  {roles.map((role) => {
                    const active = role.id === selectedRoleId;
                    const hasOverride = Boolean(scopedOverrideForRole(overrides, target, role.id));
                    return (
                      <li key={role.id}>
                        <button
                          type="button"
                          onClick={() => setSelectedRoleId(role.id)}
                          className={`flex w-full items-center gap-2 rounded-lg px-2 py-2 text-left text-[13px] transition-colors ${
                            active
                              ? 'bg-surface-active text-ink'
                              : 'text-ink-2 hover:bg-surface-hover/60 hover:text-ink'
                          }`}
                        >
                          <span
                            className="h-2.5 w-2.5 shrink-0 rounded-full"
                            style={{ background: roleColorCss(role.color) }}
                          />
                          <span className="min-w-0 flex-1 truncate font-medium">
                            {role.is_everyone ? '@everyone' : role.name}
                          </span>
                          {hasOverride ? (
                            <span className="font-mono text-[10px] text-accent">●</span>
                          ) : null}
                        </button>
                      </li>
                    );
                  })}
                </ul>
              </aside>

              <div className="min-h-0 overflow-y-auto p-5">
                {!selectedRole ? (
                  <p className="text-sm text-ink-3">Select a role to edit overrides.</p>
                ) : (
                  <div className="max-w-lg space-y-4">
                    <div>
                      <h3 className="text-base font-semibold text-ink">
                        {selectedRole.is_everyone ? '@everyone' : selectedRole.name}
                      </h3>
                      <p className="mt-1 text-sm text-ink-3">
                        {isChannel
                          ? 'Channel overrides stack on role grants and category overrides. Deny beats allow within each layer.'
                          : 'Category overrides apply to every channel in this category unless a channel override replaces them.'}
                      </p>
                    </div>

                    {inheritedSummary ? (
                      <p className="rounded-lg border border-line/50 bg-surface/50 px-3 py-2 text-[13px] text-ink-3">
                        <span className="font-medium text-ink-2">Category: </span>
                        {inheritedSummary}
                      </p>
                    ) : null}

                    {PERM_BITS.map((perm) => {
                      const value = readTri(draft, perm.family, perm.bit);
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
                            disabled={pending}
                            onChange={(next) => void setPerm(perm.family, perm.bit, next)}
                          />
                        </div>
                      );
                    })}
                  </div>
                )}
              </div>
            </div>
          )}
        </div>
      </div>
    </Portal>
  );
}
