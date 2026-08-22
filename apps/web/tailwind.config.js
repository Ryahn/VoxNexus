/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
        // Backgrounds — descending depth
        app: 'rgb(var(--bg-app) / <alpha-value>)',
        rail: 'rgb(var(--bg-rail) / <alpha-value>)',
        panel: 'rgb(var(--bg-panel) / <alpha-value>)',
        'panel-2': 'rgb(var(--bg-panel-2) / <alpha-value>)',
        surface: 'rgb(var(--surface) / <alpha-value>)',
        'surface-hover': 'rgb(var(--surface-hover) / <alpha-value>)',
        'surface-active': 'rgb(var(--surface-active) / <alpha-value>)',
        input: 'rgb(var(--input) / <alpha-value>)',
        // Text
        ink: 'rgb(var(--text) / <alpha-value>)',
        'ink-2': 'rgb(var(--text-2) / <alpha-value>)',
        'ink-3': 'rgb(var(--text-3) / <alpha-value>)',
        'ink-4': 'rgb(var(--text-4) / <alpha-value>)',
        // Lines
        line: 'rgb(var(--border) / <alpha-value>)',
        'line-2': 'rgb(var(--border-2) / <alpha-value>)',
        // Accents
        accent: 'rgb(var(--accent) / <alpha-value>)',
        'accent-2': 'rgb(var(--accent-2) / <alpha-value>)',
        magenta: 'rgb(var(--magenta) / <alpha-value>)',
        mention: 'rgb(var(--mention) / <alpha-value>)',
        // Status
        online: 'rgb(var(--online) / <alpha-value>)',
        idle: 'rgb(var(--idle) / <alpha-value>)',
        dnd: 'rgb(var(--dnd) / <alpha-value>)',
        success: 'rgb(var(--success) / <alpha-value>)',
        warning: 'rgb(var(--warning) / <alpha-value>)',
        error: 'rgb(var(--error) / <alpha-value>)',
      },
      fontFamily: {
        sans: ['Sora', 'system-ui', 'sans-serif'],
        body: ['Manrope', 'system-ui', 'sans-serif'],
        mono: ['"JetBrains Mono"', 'ui-monospace', 'monospace'],
      },
      fontSize: {
        '2xs': ['0.6875rem', { lineHeight: '1rem' }], // 11px
        '3xs': ['0.625rem', { lineHeight: '0.875rem' }], // 10px
      },
      boxShadow: {
        pop: '0 12px 40px -8px rgb(0 0 0 / 0.55), 0 2px 8px -2px rgb(0 0 0 / 0.4)',
        panel: '0 8px 30px -10px rgb(0 0 0 / 0.5)',
        'accent-glow':
          '0 0 0 1px rgb(var(--accent) / 0.25), 0 0 18px -6px rgb(var(--accent) / 0.4)',
        'inset-line': 'inset 0 0 0 1px rgb(var(--border) / 0.6)',
      },
      transitionTimingFunction: {
        swift: 'cubic-bezier(0.22, 1, 0.36, 1)',
      },
      keyframes: {
        'pop-in': {
          '0%': { opacity: '0', transform: 'translateY(4px) scale(0.98)' },
          '100%': { opacity: '1', transform: 'translateY(0) scale(1)' },
        },
        'fade-in': {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        'slide-in-right': {
          '0%': { opacity: '0', transform: 'translateX(12px)' },
          '100%': { opacity: '1', transform: 'translateX(0)' },
        },
        'pulse-ring': {
          '0%': { boxShadow: '0 0 0 0 rgb(var(--online) / 0.5)' },
          '70%': { boxShadow: '0 0 0 5px rgb(var(--online) / 0)' },
          '100%': { boxShadow: '0 0 0 0 rgb(var(--online) / 0)' },
        },
      },
      animation: {
        'pop-in': 'pop-in 0.14s cubic-bezier(0.22,1,0.36,1)',
        'fade-in': 'fade-in 0.12s ease-out',
        'slide-in-right': 'slide-in-right 0.16s cubic-bezier(0.22,1,0.36,1)',
        'pulse-ring': 'pulse-ring 2.4s ease-out infinite',
      },
    },
  },
  plugins: [],
};
