import { GitBranch, MoreHorizontal, Pencil, Reply, SmilePlus, Trash2 } from 'lucide-react';
import { messageMenu } from '../lib/menus';
import { useUI } from '../store';
import type { Message } from '../types';
import { Tooltip } from './ui/Tooltip';

export function MessageActions({
  message,
  canEdit,
  canDelete,
  onEdit,
  onDelete,
}: {
  message: Message;
  canEdit?: boolean;
  canDelete?: boolean;
  onEdit?: () => void;
  onDelete?: () => void;
}) {
  const setReplyingTo = useUI((s) => s.setReplyingTo);
  const setThreadOpen = useUI((s) => s.setThreadOpen);
  const openMenu = useUI((s) => s.openMenu);
  const isMine = canEdit ?? message.authorId === 'me';
  const showDelete = canDelete ?? isMine;

  return (
    <div className="absolute -top-3 right-3 z-10 hidden items-center gap-0.5 rounded-lg border border-line-2/70 bg-surface p-0.5 shadow-pop group-hover/msg:flex">
      <Act icon={SmilePlus} label="Add Reaction" />
      <Act icon={Reply} label="Reply" onClick={() => setReplyingTo(message.id)} />
      <Act icon={GitBranch} label="Start Thread" onClick={() => setThreadOpen(true)} />
      {isMine && <Act icon={Pencil} label="Edit" onClick={onEdit} />}
      {showDelete && <Act icon={Trash2} label="Delete" onClick={onDelete} />}
      <Act
        icon={MoreHorizontal}
        label="More"
        onClick={(e) => {
          const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
          openMenu({ x: r.right, y: r.bottom + 4, left: true }, messageMenu());
        }}
      />
    </div>
  );
}

function Act({
  icon: Icon,
  label,
  onClick,
}: {
  icon: typeof Reply;
  label: string;
  onClick?: (e: React.MouseEvent) => void;
}) {
  return (
    <Tooltip label={label} side="top">
      <button
        type="button"
        aria-label={label}
        onClick={onClick}
        className="grid h-7 w-7 place-items-center rounded-md text-ink-2 transition-colors hover:bg-surface-hover hover:text-ink"
      >
        <Icon size={16} strokeWidth={1.9} />
      </button>
    </Tooltip>
  );
}
