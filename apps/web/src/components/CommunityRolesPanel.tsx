import {
  assignMemberRole,
  createRole,
  deleteRole,
  listCommunityMembers,
  listMemberRoles,
  listRoles,
  type RoleResponse,
  removeMemberRole,
} from '@voxnexus/api-client';
import { useCallback, useEffect, useState } from 'react';
import { readApiErrorMessage } from '../lib/apiError';

type Props = {
  communityId: string;
  canManage: boolean;
};

export function CommunityRolesPanel({ communityId, canManage }: Props) {
  const [roles, setRoles] = useState<RoleResponse[]>([]);
  const [members, setMembers] = useState<{ id: string; name: string }[]>([]);
  const [selectedMember, setSelectedMember] = useState('');
  const [memberRoleIds, setMemberRoleIds] = useState<string[]>([]);
  const [newRoleName, setNewRoleName] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);

  const refreshRoles = useCallback(async () => {
    const result = await listRoles({ path: { community_id: communityId } });
    if (result.error || !result.data) {
      setError(readApiErrorMessage(result.error, 'Could not load roles.'));
      return;
    }
    setRoles(result.data.roles);
  }, [communityId]);

  const refreshMembers = useCallback(async () => {
    const result = await listCommunityMembers({
      path: { community_id: communityId },
      query: { limit: 100 },
    });
    if (result.data?.items) {
      setMembers(
        result.data.items.map((m) => ({
          id: m.account_id,
          name: m.nickname.trim() || m.display_name || 'Member',
        })),
      );
    }
  }, [communityId]);

  useEffect(() => {
    void refreshRoles();
    void refreshMembers();
  }, [refreshRoles, refreshMembers]);

  const loadMemberRoles = async (accountId: string) => {
    setSelectedMember(accountId);
    if (!accountId) {
      setMemberRoleIds([]);
      return;
    }
    const result = await listMemberRoles({
      path: { community_id: communityId, account_id: accountId },
    });
    if (result.data) {
      setMemberRoleIds(result.data.roles.map((role) => role.id));
    }
  };

  const create = async () => {
    const name = newRoleName.trim();
    if (!name) return;
    setPending(true);
    setError(null);
    const result = await createRole({
      path: { community_id: communityId },
      body: { name },
    });
    setPending(false);
    if (result.error) {
      setError(readApiErrorMessage(result.error, 'Could not create role.'));
      return;
    }
    setNewRoleName('');
    await refreshRoles();
  };

  const toggleMemberRole = async (roleId: string, assigned: boolean) => {
    if (!selectedMember || !canManage) return;
    setPending(true);
    setError(null);
    const result = assigned
      ? await removeMemberRole({
          path: {
            community_id: communityId,
            account_id: selectedMember,
            role_id: roleId,
          },
        })
      : await assignMemberRole({
          path: { community_id: communityId, account_id: selectedMember },
          body: { role_id: roleId },
        });
    setPending(false);
    if (result.error) {
      setError(readApiErrorMessage(result.error, 'Could not update member roles.'));
      return;
    }
    await loadMemberRoles(selectedMember);
  };

  const customRoles = roles.filter((role) => !role.is_everyone);

  return (
    <div className="space-y-4">
      <div>
        <h3 className="mb-2 text-xs font-medium uppercase tracking-wide text-ink-3">Roles</h3>
        <ul className="space-y-1">
          {roles.map((role) => (
            <li
              key={role.id}
              className="flex items-center justify-between rounded-lg border border-line/50 px-3 py-2 text-sm"
            >
              <span style={{ color: `rgb(${role.color})` }}>{role.name}</span>
              {canManage && !role.is_everyone ? (
                <button
                  type="button"
                  disabled={pending}
                  onClick={() =>
                    void (async () => {
                      setPending(true);
                      await deleteRole({ path: { role_id: role.id } });
                      setPending(false);
                      await refreshRoles();
                    })()
                  }
                  className="text-xs text-dnd hover:underline disabled:opacity-50"
                >
                  Delete
                </button>
              ) : null}
            </li>
          ))}
        </ul>
        {canManage ? (
          <div className="mt-2 flex gap-2">
            <input
              value={newRoleName}
              onChange={(e) => setNewRoleName(e.target.value)}
              placeholder="New role name"
              className="flex-1 rounded-lg border border-line-2/80 bg-surface px-3 py-2 text-sm"
            />
            <button
              type="button"
              disabled={pending}
              onClick={() => void create()}
              className="rounded-lg bg-accent px-3 py-2 text-sm font-medium text-app disabled:opacity-60"
            >
              Add
            </button>
          </div>
        ) : null}
      </div>

      {canManage ? (
        <div>
          <h3 className="mb-2 text-xs font-medium uppercase tracking-wide text-ink-3">
            Assign roles
          </h3>
          <select
            value={selectedMember}
            onChange={(e) => void loadMemberRoles(e.target.value)}
            className="mb-2 w-full rounded-lg border border-line-2/80 bg-surface px-3 py-2 text-sm"
          >
            <option value="">Pick a member…</option>
            {members.map((member) => (
              <option key={member.id} value={member.id}>
                {member.name}
              </option>
            ))}
          </select>
          {selectedMember ? (
            <ul className="space-y-1">
              {customRoles.map((role) => {
                const assigned = memberRoleIds.includes(role.id);
                return (
                  <li key={role.id}>
                    <label className="flex cursor-pointer items-center gap-2 rounded px-2 py-1 text-sm">
                      <input
                        type="checkbox"
                        checked={assigned}
                        disabled={pending}
                        onChange={() => void toggleMemberRole(role.id, assigned)}
                      />
                      <span style={{ color: `rgb(${role.color})` }}>{role.name}</span>
                    </label>
                  </li>
                );
              })}
            </ul>
          ) : null}
        </div>
      ) : null}

      {error ? <p className="text-sm text-dnd">{error}</p> : null}
    </div>
  );
}
