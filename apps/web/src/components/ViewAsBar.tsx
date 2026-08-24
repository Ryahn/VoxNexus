import {
  listCommunityMembers,
  listRoles,
  type RoleResponse,
  viewAsChannels,
} from '@voxnexus/api-client';
import { Eye, X } from 'lucide-react';
import { useEffect, useState } from 'react';
import { readApiErrorMessage } from '../lib/apiError';
import { useUI, type ViewAsSession } from '../store';

type Props = {
  communityId: string;
  spaceId: string;
  canManage: boolean;
  onSimulatedChannels: (channelIds: string[] | null, label: string | null) => void;
};

type ModeChoice = 'off' | 'visitor' | 'member' | 'roles';

export function ViewAsBar({ communityId, spaceId, canManage, onSimulatedChannels }: Props) {
  const viewAsOpen = useUI((s) => s.viewAsOpen);
  const setViewAsOpen = useUI((s) => s.setViewAsOpen);
  const viewAs = useUI((s) => s.viewAs);
  const setViewAs = useUI((s) => s.setViewAs);

  const [mode, setMode] = useState<ModeChoice>('off');
  const [memberId, setMemberId] = useState('');
  const [roleId, setRoleId] = useState('');
  const [members, setMembers] = useState<{ id: string; label: string }[]>([]);
  const [roles, setRoles] = useState<RoleResponse[]>([]);
  const [label, setLabel] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);

  useEffect(() => {
    if (!canManage || (!viewAsOpen && !viewAs)) return;
    void (async () => {
      const [membersPage, rolesResult] = await Promise.all([
        listCommunityMembers({ path: { community_id: communityId }, query: { limit: 100 } }),
        listRoles({ path: { community_id: communityId } }),
      ]);
      if (membersPage.data?.items) {
        setMembers(
          membersPage.data.items.map((m) => ({
            id: m.account_id,
            label: m.nickname.trim() || m.display_name.trim() || m.account_id,
          })),
        );
      }
      if (rolesResult.data?.roles) {
        const sorted = [...rolesResult.data.roles].sort((a, b) => a.weight - b.weight);
        setRoles(sorted);
        const firstCustom = sorted.find((role) => !role.is_everyone);
        if (firstCustom && !roleId) setRoleId(firstCustom.id);
      }
    })();
  }, [canManage, communityId, viewAs, viewAsOpen, roleId]);

  useEffect(() => {
    if (!viewAs) {
      setMode('off');
      setLabel(null);
      onSimulatedChannels(null, null);
      return;
    }
    setMode(viewAs.mode);
    if (viewAs.mode === 'member') setMemberId(viewAs.accountId);
    if (viewAs.mode === 'roles' && viewAs.roleIds[0]) setRoleId(viewAs.roleIds[0]);
    void runSimulation(viewAs);
    // eslint-disable-next-line react-hooks/exhaustive-deps -- refresh when session/space changes
  }, [viewAs, communityId, spaceId]);

  const runSimulation = async (session: ViewAsSession) => {
    setPending(true);
    setError(null);
    const body =
      session.mode === 'visitor'
        ? { community_id: communityId, space_id: spaceId, mode: 'visitor' as const, role_ids: [] }
        : session.mode === 'member'
          ? {
              community_id: communityId,
              space_id: spaceId,
              mode: 'member' as const,
              account_id: session.accountId,
              role_ids: [],
            }
          : {
              community_id: communityId,
              space_id: spaceId,
              mode: 'roles' as const,
              role_ids: session.roleIds,
            };
    const result = await viewAsChannels({ body });
    setPending(false);
    if (result.error || !result.data) {
      setError(readApiErrorMessage(result.error, 'Could not simulate channel list.'));
      onSimulatedChannels(null, null);
      return;
    }
    setLabel(result.data.label);
    onSimulatedChannels(
      result.data.channels.map((ch) => ch.id),
      result.data.label,
    );
  };

  const apply = () => {
    if (mode === 'off') {
      setViewAs(null);
      setViewAsOpen(false);
      return;
    }
    if (mode === 'visitor') {
      setViewAs({ mode: 'visitor' });
      return;
    }
    if (mode === 'member') {
      if (!memberId) {
        setError('Pick a member.');
        return;
      }
      setViewAs({ mode: 'member', accountId: memberId });
      return;
    }
    if (!roleId) {
      setError('Pick a role.');
      return;
    }
    setViewAs({ mode: 'roles', roleIds: [roleId] });
  };

  if (!canManage || (!viewAsOpen && !viewAs)) return null;

  return (
    <div className="mx-2 mb-2 rounded-md border border-line/80 bg-surface-hover/40 px-2 py-2">
      <div className="mb-1.5 flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-wide text-ink-3">
        <Eye size={12} className="text-accent" />
        View As
        <button
          type="button"
          aria-label="Close View As"
          className="ml-auto rounded p-0.5 text-ink-4 hover:bg-surface-hover hover:text-ink"
          onClick={() => {
            setViewAs(null);
            setViewAsOpen(false);
          }}
        >
          <X size={12} />
        </button>
      </div>
      <p className="mb-2 text-[11px] leading-snug text-ink-4">
        Sidebar preview only. Creates and settings still use your real permissions.
      </p>
      <div className="flex flex-col gap-1.5">
        <label className="block text-[11px] text-ink-3">
          Mode
          <select
            className="mt-0.5 w-full rounded border border-line-2/60 bg-input px-1.5 py-1 text-[12px] text-ink"
            value={mode}
            onChange={(e) => setMode(e.target.value as ModeChoice)}
          >
            <option value="off">Off</option>
            <option value="visitor">Visitor</option>
            <option value="member">Member</option>
            <option value="roles">Role</option>
          </select>
        </label>
        {mode === 'member' ? (
          <label className="block text-[11px] text-ink-3">
            Member
            <select
              className="mt-0.5 w-full rounded border border-line-2/60 bg-input px-1.5 py-1 text-[12px] text-ink"
              value={memberId}
              onChange={(e) => setMemberId(e.target.value)}
            >
              <option value="">Select…</option>
              {members.map((m) => (
                <option key={m.id} value={m.id}>
                  {m.label}
                </option>
              ))}
            </select>
          </label>
        ) : null}
        {mode === 'roles' ? (
          <label className="block text-[11px] text-ink-3">
            Role (+ @everyone)
            <select
              className="mt-0.5 w-full rounded border border-line-2/60 bg-input px-1.5 py-1 text-[12px] text-ink"
              value={roleId}
              onChange={(e) => setRoleId(e.target.value)}
            >
              {roles
                .filter((role) => !role.is_everyone)
                .map((role) => (
                  <option key={role.id} value={role.id}>
                    {role.name}
                  </option>
                ))}
              <option value="">@everyone only</option>
            </select>
          </label>
        ) : null}
        <button
          type="button"
          disabled={pending}
          onClick={apply}
          className="rounded bg-accent/90 px-2 py-1 text-[12px] font-medium text-app hover:bg-accent disabled:opacity-50"
        >
          {pending ? 'Updating…' : mode === 'off' ? 'Exit' : 'Apply'}
        </button>
      </div>
      {label ? (
        <p className="mt-1.5 text-[11px] text-accent">
          Viewing as <span className="font-medium">{label}</span>
        </p>
      ) : null}
      {error ? <p className="mt-1 text-[11px] text-dnd">{error}</p> : null}
    </div>
  );
}
