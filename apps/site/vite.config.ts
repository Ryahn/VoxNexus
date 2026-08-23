import { copyFileSync, readFileSync, writeFileSync } from 'node:fs';
import type { IncomingMessage, ServerResponse } from 'node:http';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import react from '@vitejs/plugin-react';
import { defineConfig, type Plugin } from 'vite';

const rootDir = fileURLToPath(new URL('.', import.meta.url));
const repoRoot = resolve(rootDir, '../..');
const outDir = resolve(repoRoot, 'docs');
const openapiPath = resolve(repoRoot, 'packages/api-client/openapi.json');

function openApiPlugin(): Plugin {
  const serveOpenApi = (_req: IncomingMessage, res: ServerResponse, next: () => void) => {
    if (_req.url !== '/openapi.json') {
      next();
      return;
    }
    res.setHeader('Content-Type', 'application/json');
    res.end(readFileSync(openapiPath));
  };

  return {
    name: 'openapi',
    configureServer(server) {
      server.middlewares.use(serveOpenApi);
    },
    configurePreviewServer(server) {
      server.middlewares.use(serveOpenApi);
    },
    writeBundle() {
      copyFileSync(openapiPath, resolve(outDir, 'openapi.json'));
      const indexPath = resolve(outDir, 'index.html');
      const index = readFileSync(indexPath, 'utf8');
      writeFileSync(resolve(outDir, '404.html'), index);
    },
  };
}

export default defineConfig({
  plugins: [react(), openApiPlugin()],
  resolve: {
    alias: {
      '@': resolve(rootDir, 'src'),
    },
  },
  base: '/',
  build: {
    outDir,
    emptyOutDir: true,
  },
  server: {
    port: 5174,
    host: true,
  },
});
