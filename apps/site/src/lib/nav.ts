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
      { label: 'Architecture', to: '/docs/guides/architecture' },
      { label: 'Docker Compose', to: '/docs/setup/compose' },
      { label: 'Configuration', to: '/docs/setup/config' },
      { label: 'Developing', to: '/docs/guides/develop' },
    ],
  },
  {
    title: 'Product',
    items: [
      { label: 'Authentication', to: '/docs/guides/auth' },
      { label: 'OIDC / SSO', to: '/docs/guides/oidc' },
      { label: 'Profiles & presence', to: '/docs/guides/profiles' },
      { label: 'Instance', to: '/docs/guides/instance' },
      { label: 'Communities', to: '/docs/guides/communities' },
      { label: 'Invites', to: '/docs/guides/invites' },
      { label: 'Spaces', to: '/docs/guides/spaces' },
      { label: 'Channels', to: '/docs/guides/channels' },
      { label: 'Roles', to: '/docs/guides/roles' },
      { label: 'Permissions', to: '/docs/guides/permissions' },
    ],
  },
  {
    title: 'API',
    items: [
      { label: 'HTTP reference', to: '/docs/api' },
      { label: 'API conventions', to: '/docs/guides/api-conventions' },
      { label: 'Gateway', to: '/docs/guides/gateway' },
    ],
  },
  {
    title: 'Platform',
    items: [
      { label: 'Database', to: '/docs/guides/database' },
      { label: 'Storage', to: '/docs/guides/storage' },
      { label: 'Search', to: '/docs/guides/search' },
      { label: 'Jobs', to: '/docs/guides/jobs' },
      { label: 'Observability', to: '/docs/guides/observability' },
      { label: 'Codegen', to: '/docs/guides/codegen' },
      { label: 'CI', to: '/docs/guides/ci' },
    ],
  },
];
