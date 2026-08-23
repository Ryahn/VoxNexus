import apiMd from '../../content/api.md?raw';
import authMd from '../../content/auth.md?raw';
import ciMd from '../../content/ci.md?raw';
import codegenMd from '../../content/codegen.md?raw';
import composeMd from '../../content/compose.md?raw';
import configMd from '../../content/config.md?raw';
import databaseMd from '../../content/database.md?raw';
import gatewayMd from '../../content/gateway.md?raw';
import jobsMd from '../../content/jobs.md?raw';
import observabilityMd from '../../content/observability.md?raw';
import searchMd from '../../content/search.md?raw';
import storageMd from '../../content/storage.md?raw';

export type DocEntry = {
  slug: string;
  title: string;
  body: string;
};

export const docPages: Record<string, DocEntry> = {
  'setup/compose': { slug: 'setup/compose', title: 'Docker Compose', body: composeMd },
  'setup/config': { slug: 'setup/config', title: 'Configuration', body: configMd },
  'guides/api-conventions': {
    slug: 'guides/api-conventions',
    title: 'API conventions',
    body: apiMd,
  },
  'guides/auth': { slug: 'guides/auth', title: 'Authentication', body: authMd },
  'guides/codegen': { slug: 'guides/codegen', title: 'Codegen', body: codegenMd },
  'guides/gateway': { slug: 'guides/gateway', title: 'Gateway', body: gatewayMd },
  'guides/observability': {
    slug: 'guides/observability',
    title: 'Observability',
    body: observabilityMd,
  },
  'guides/storage': { slug: 'guides/storage', title: 'Storage', body: storageMd },
  'guides/search': { slug: 'guides/search', title: 'Search', body: searchMd },
  'guides/jobs': { slug: 'guides/jobs', title: 'Jobs', body: jobsMd },
  'guides/database': { slug: 'guides/database', title: 'Database', body: databaseMd },
  'guides/ci': { slug: 'guides/ci', title: 'CI', body: ciMd },
};

export function getDocPage(slug: string): DocEntry | undefined {
  return docPages[slug];
}
