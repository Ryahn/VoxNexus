export type NavItem = {
  label: string;
  to: string;
};

export type NavSection = {
  title: string;
  items: NavItem[];
};

export const docsNav: NavSection[] = [
  {
    title: 'Getting started',
    items: [
      { label: 'Overview', to: '/docs' },
      { label: 'Docker Compose', to: '/docs/setup/compose' },
      { label: 'Configuration', to: '/docs/setup/config' },
    ],
  },
  {
    title: 'API',
    items: [{ label: 'HTTP reference', to: '/docs/api' }],
  },
  {
    title: 'Guides',
    items: [
      { label: 'API conventions', to: '/docs/guides/api-conventions' },
      { label: 'Authentication', to: '/docs/guides/auth' },
      { label: 'Gateway', to: '/docs/guides/gateway' },
      { label: 'Codegen', to: '/docs/guides/codegen' },
      { label: 'Observability', to: '/docs/guides/observability' },
      { label: 'Storage', to: '/docs/guides/storage' },
      { label: 'Search', to: '/docs/guides/search' },
      { label: 'Jobs', to: '/docs/guides/jobs' },
      { label: 'Database', to: '/docs/guides/database' },
      { label: 'CI', to: '/docs/guides/ci' },
    ],
  },
];
