/* Deterministic generated avatars — no network, stable per seed.
   Produces a two-stop gradient + a faint geometric overlay so
   avatars read as distinct "identity chips" in the cyberpunk key. */

function hash(str: string): number {
  let h = 2166136261;
  for (let i = 0; i < str.length; i++) {
    h ^= str.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return h >>> 0;
}

const PALETTE: [number, number, number][] = [
  [54, 210, 205],
  [138, 124, 246],
  [240, 97, 168],
  [99, 202, 130],
  [99, 179, 237],
  [240, 180, 41],
  [96, 165, 250],
  [244, 114, 182],
];

export interface AvatarStyle {
  gradient: string;
  ring: string;
  initials: string;
  angle: number;
}

export function avatarStyle(seed: string, label: string): AvatarStyle {
  const h = hash(seed);
  const a = PALETTE[h % PALETTE.length];
  const b = PALETTE[(h >>> 8) % PALETTE.length];
  const angle = (h >>> 3) % 360;
  const initials = label
    .split(/\s+/)
    .slice(0, 2)
    .map((w) => w[0])
    .join('')
    .toUpperCase();
  return {
    gradient: `linear-gradient(${angle}deg, rgb(${a.join(' ')}) 0%, rgb(${b.join(' ')}) 100%)`,
    ring: `rgb(${a.join(' ')} / 0.5)`,
    initials,
    angle,
  };
}

export function bannerGradient(seed = 'x'): string {
  const h = hash(seed);
  const a = PALETTE[h % PALETTE.length];
  const b = PALETTE[(h >>> 5) % PALETTE.length];
  const angle = (h >>> 2) % 360;
  return `linear-gradient(${angle}deg, rgb(${a.join(' ')} / 0.9), rgb(${b.join(' ')} / 0.55))`;
}
