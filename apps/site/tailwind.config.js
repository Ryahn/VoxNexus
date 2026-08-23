import voxPreset from '@voxnexus/ui/theme/tailwind-preset';

/** @type {import('tailwindcss').Config} */
export default {
  presets: [voxPreset],
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  plugins: [],
};
