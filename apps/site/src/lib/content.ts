import apiMd from '../../content/api.md?raw';
import architectureMd from '../../content/architecture.md?raw';
import authMd from '../../content/auth.md?raw';
import channelsMd from '../../content/channels.md?raw';
import ciMd from '../../content/ci.md?raw';
import codegenMd from '../../content/codegen.md?raw';
import communitiesMd from '../../content/communities.md?raw';
import composeMd from '../../content/compose.md?raw';
import configMd from '../../content/config.md?raw';
import databaseMd from '../../content/database.md?raw';
import developMd from '../../content/develop.md?raw';
import formattingMd from '../../content/formatting.md?raw';
import gatewayMd from '../../content/gateway.md?raw';
import instanceMd from '../../content/instance.md?raw';
import invitesMd from '../../content/invites.md?raw';
import jobsMd from '../../content/jobs.md?raw';
import observabilityMd from '../../content/observability.md?raw';
import oidcMd from '../../content/oidc.md?raw';
import permissionsMd from '../../content/permissions.md?raw';
import profilesMd from '../../content/profiles.md?raw';
import rolesMd from '../../content/roles.md?raw';
import searchMd from '../../content/search.md?raw';
import spacesMd from '../../content/spaces.md?raw';
import storageMd from '../../content/storage.md?raw';

export type DocEntry = {
  slug: string;
  title: string;
  body: string;
};

export const docPages: Record<string, DocEntry> = {
  'setup/compose': { slug: 'setup/compose', title: 'Docker Compose', body: composeMd },
  'setup/config': { slug: 'setup/config', title: 'Configuration', body: configMd },
  'guides/architecture': {
    slug: 'guides/architecture',
    title: 'Architecture',
    body: architectureMd,
  },
  'guides/develop': { slug: 'guides/develop', title: 'Developing', body: developMd },
  'guides/api-conventions': {
    slug: 'guides/api-conventions',
    title: 'API conventions',
    body: apiMd,
  },
  'guides/auth': { slug: 'guides/auth', title: 'Authentication', body: authMd },
  'guides/oidc': { slug: 'guides/oidc', title: 'OIDC / SSO', body: oidcMd },
  'guides/profiles': { slug: 'guides/profiles', title: 'Profiles & presence', body: profilesMd },
  'guides/instance': { slug: 'guides/instance', title: 'Instance settings', body: instanceMd },
  'guides/communities': { slug: 'guides/communities', title: 'Communities', body: communitiesMd },
  'guides/invites': { slug: 'guides/invites', title: 'Invites', body: invitesMd },
  'guides/spaces': { slug: 'guides/spaces', title: 'Spaces', body: spacesMd },
  'guides/channels': { slug: 'guides/channels', title: 'Categories & channels', body: channelsMd },
  'guides/formatting': {
    slug: 'guides/formatting',
    title: 'Message formatting',
    body: formattingMd,
  },
  'guides/roles': { slug: 'guides/roles', title: 'Roles', body: rolesMd },
  'guides/permissions': { slug: 'guides/permissions', title: 'Permissions', body: permissionsMd },
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
