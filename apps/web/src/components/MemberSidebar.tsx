import { useMemo } from 'react';
import { buildMemberSections } from '../data/members';
import { MemberItem } from './MemberItem';

export function MemberSidebar() {
  const sections = useMemo(() => buildMemberSections(), []);

  return (
    <aside className="flex h-full w-60 shrink-0 flex-col border-l border-line/70 bg-panel">
      <div className="flex h-12 shrink-0 items-center border-b border-line/70 px-3">
        <span className="font-sans text-[11px] font-semibold uppercase tracking-[0.14em] text-ink-3">
          Members
        </span>
      </div>
      <div className="min-h-0 flex-1 overflow-y-auto px-2 py-2">
        {sections.map((sec) => (
          <div key={sec.key} className="mb-3">
            <div className="flex items-center gap-1.5 px-2 pb-1">
              <span
                className="kicker"
                style={sec.color ? { color: `rgb(${sec.color})` } : undefined}
              >
                {sec.label}
              </span>
              <span className="font-mono text-3xs text-ink-4">— {sec.members.length}</span>
            </div>
            <div className="flex flex-col gap-px">
              {sec.members.map((u) => (
                <MemberItem key={u.id} user={u} />
              ))}
            </div>
          </div>
        ))}
      </div>
    </aside>
  );
}
