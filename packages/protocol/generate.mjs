import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { compile } from 'json-schema-to-typescript';

const root = path.dirname(fileURLToPath(import.meta.url));
const schemaPath = path.join(root, 'gateway.schema.json');
const outDir = path.join(root, 'src', 'generated');
const outPath = path.join(outDir, 'gateway.ts');

const schema = JSON.parse(await readFile(schemaPath, 'utf8'));
const banner = `/* eslint-disable */
/**
 * This file was automatically generated from gateway.schema.json.
 * DO NOT MODIFY IT BY HAND. Run \`pnpm codegen\` instead.
 */
`;
const types = await compile(schema, 'GatewaySchemaCatalog', {
  bannerComment: banner,
  cwd: root,
  style: {
    singleQuote: true,
  },
  unreachableDefinitions: true,
});

await mkdir(outDir, { recursive: true });
await writeFile(outPath, types, 'utf8');
console.log(`wrote ${outPath}`);
