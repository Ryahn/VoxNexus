import type { TriState } from '../lib/rolePermissions';

const TRI_STATES: { id: TriState; label: string }[] = [
  { id: 'inherit', label: 'Inherit' },
  { id: 'allow', label: 'Allow' },
  { id: 'deny', label: 'Deny' },
];

function triStateButtonClass(active: boolean, state: TriState): string {
  if (!active) {
    return 'text-ink-3 hover:bg-surface-hover/40 hover:text-ink-2';
  }
  if (state === 'allow') return 'bg-emerald-500/20 text-emerald-300';
  if (state === 'deny') return 'bg-red-500/20 text-red-300';
  return 'bg-accent/15 text-ink';
}

type Props = {
  value: TriState;
  disabled?: boolean;
  onChange: (next: TriState) => void;
};

export function TriStateToggle({ value, disabled = false, onChange }: Props) {
  return (
    <div
      className="inline-flex shrink-0 divide-x divide-line/60 overflow-hidden rounded-lg border border-line/60"
      role="group"
      aria-label="Permission state"
    >
      {TRI_STATES.map((opt) => (
        <button
          key={opt.id}
          type="button"
          disabled={disabled}
          aria-pressed={value === opt.id}
          onClick={() => onChange(opt.id)}
          className={`px-2.5 py-1 text-[12px] font-medium transition-colors disabled:cursor-not-allowed disabled:opacity-60 ${triStateButtonClass(value === opt.id, opt.id)}`}
        >
          {opt.label}
        </button>
      ))}
    </div>
  );
}
