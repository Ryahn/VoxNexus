import {
  AtSign,
  Check,
  Circle,
  CornerDownLeft,
  FileText,
  Hash,
  Images,
  type LucideIcon,
  MessageSquare,
  Palette,
  Search,
  User,
  Users2,
  Volume2,
} from 'lucide-react';
import { useEffect, useMemo, useRef, useState } from 'react';
import { quickActions, searchIndex } from '../data/search';
import { useUI } from '../store';
import type { SearchResult } from '../types';
import { Portal } from './ui/Portal';

const glyphs: Record<string, LucideIcon> = {
  hash: Hash,
  at: AtSign,
  check: Check,
  circle: Circle,
  palette: Palette,
  message: MessageSquare,
  user: User,
  file: FileText,
  text: Hash,
  voice: Volume2,
  media: Images,
  community: Users2,
};

const kindLabel: Record<string, string> = {
  command: 'Actions',
  channel: 'Channels',
  user: 'People',
  message: 'Messages',
  file: 'Files',
  community: 'Communities',
};

export function SearchPalette() {
  const open = useUI((s) => s.searchOpen);
  const setOpen = useUI((s) => s.setSearchOpen);
  const [q, setQ] = useState('');
  const [active, setActive] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (open) {
      setQ('');
      setActive(0);
      setTimeout(() => inputRef.current?.focus(), 20);
    }
  }, [open]);

  const results = useMemo(() => {
    const pool: SearchResult[] = q ? searchIndex : [...quickActions, ...searchIndex.slice(0, 4)];
    if (!q) return pool;
    const needle = q.toLowerCase();
    return pool.filter(
      (r) => r.title.toLowerCase().includes(needle) || r.subtitle?.toLowerCase().includes(needle),
    );
  }, [q]);

  // group by kind, preserving encounter order
  const groups = useMemo(() => {
    const map = new Map<string, SearchResult[]>();
    for (const r of results) {
      if (!map.has(r.kind)) map.set(r.kind, []);
      map.get(r.kind)!.push(r);
    }
    return Array.from(map.entries());
  }, [results]);

  const flat = results;
  useEffect(() => {
    setActive((a) => Math.min(a, Math.max(0, flat.length - 1)));
  }, [flat.length]);

  if (!open) return null;

  const onKey = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') setOpen(false);
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setActive((a) => (a + 1) % flat.length);
    }
    if (e.key === 'ArrowUp') {
      e.preventDefault();
      setActive((a) => (a - 1 + flat.length) % flat.length);
    }
    if (e.key === 'Enter') {
      e.preventDefault();
      setOpen(false);
    }
  };

  let runningIndex = -1;

  return (
    <Portal>
      <div
        className="fixed inset-0 z-[800] flex items-start justify-center bg-app/70 pt-[12vh] backdrop-blur-sm animate-fade-in"
        onMouseDown={() => setOpen(false)}
      >
        <div
          className="w-[min(640px,92vw)] overflow-hidden rounded-xl border border-line-2/70 bg-panel-2 shadow-pop animate-pop-in"
          onMouseDown={(e) => e.stopPropagation()}
          onKeyDown={onKey}
        >
          {/* input */}
          <div className="flex items-center gap-3 border-b border-line/70 px-4 py-3">
            <Search size={18} className="text-ink-3" />
            <input
              ref={inputRef}
              value={q}
              onChange={(e) => setQ(e.target.value)}
              placeholder="Search messages, people, channels, files…"
              className="flex-1 bg-transparent font-body text-[15px] text-ink outline-none placeholder:text-ink-4"
            />
            <kbd className="rounded border border-line-2/60 bg-app px-1.5 py-0.5 font-mono text-3xs text-ink-3">
              Esc
            </kbd>
          </div>

          {/* results */}
          <div className="max-h-[52vh] overflow-y-auto p-2">
            {flat.length === 0 && (
              <div className="px-3 py-10 text-center text-[13px] text-ink-3">
                No results for “<span className="text-ink">{q}</span>”
              </div>
            )}
            {groups.map(([kind, items]) => (
              <div key={kind} className="mb-2">
                <div className="px-2 pb-1 pt-1">
                  <span className="kicker">{kindLabel[kind] ?? kind}</span>
                </div>
                {items.map((r) => {
                  runningIndex++;
                  const idx = runningIndex;
                  const Icon = glyphs[r.glyph ?? 'message'] ?? MessageSquare;
                  const isActive = idx === active;
                  return (
                    <button
                      key={r.id}
                      onMouseEnter={() => setActive(idx)}
                      onClick={() => setOpen(false)}
                      className={`flex w-full items-center gap-3 rounded-lg px-2.5 py-2 text-left transition-colors ${
                        isActive ? 'bg-surface-active' : 'hover:bg-surface-hover/60'
                      }`}
                    >
                      <span
                        className="grid h-8 w-8 shrink-0 place-items-center rounded-md border border-line-2/50"
                        style={{
                          color: r.accent ? `rgb(${r.accent})` : 'rgb(var(--text-2))',
                          background: r.accent
                            ? `rgb(${r.accent} / 0.1)`
                            : 'rgb(var(--surface) / 0.6)',
                        }}
                      >
                        <Icon size={16} strokeWidth={1.9} />
                      </span>
                      <span className="min-w-0 flex-1 leading-tight">
                        <span className="block truncate text-[13.5px] font-medium text-ink">
                          {r.title}
                        </span>
                        {r.subtitle && (
                          <span className="block truncate font-mono text-3xs text-ink-3">
                            {r.subtitle}
                          </span>
                        )}
                      </span>
                      {r.meta && <span className="font-mono text-3xs text-ink-4">{r.meta}</span>}
                      {isActive && <CornerDownLeft size={14} className="text-accent" />}
                    </button>
                  );
                })}
              </div>
            ))}
          </div>

          {/* footer */}
          <div className="flex items-center gap-4 border-t border-line/70 bg-panel px-4 py-2 font-mono text-3xs text-ink-4">
            <span className="flex items-center gap-1">
              <kbd className="rounded border border-line-2/50 bg-app px-1 py-px">↑↓</kbd> navigate
            </span>
            <span className="flex items-center gap-1">
              <kbd className="rounded border border-line-2/50 bg-app px-1 py-px">↵</kbd> open
            </span>
            <span className="ml-auto flex items-center gap-1.5 text-ink-3">
              <span className="h-1.5 w-1.5 rounded-full bg-accent" /> VOX Search
            </span>
          </div>
        </div>
      </div>
    </Portal>
  );
}
