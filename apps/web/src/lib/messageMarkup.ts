/** Discord-like chat markup subset (F040). Stored as plain text; rendered client-side. */

export type InlineNode =
  | { type: 'text'; text: string }
  | { type: 'everyone' }
  | { type: 'here' }
  | { type: 'user'; id: string }
  | { type: 'role'; id: string }
  | { type: 'code'; text: string }
  | { type: 'bold'; children: InlineNode[] }
  | { type: 'italic'; children: InlineNode[] }
  | { type: 'strike'; children: InlineNode[] }
  | { type: 'spoiler'; children: InlineNode[] };

export type BlockNode =
  | { type: 'paragraph'; children: InlineNode[] }
  | { type: 'codeblock'; lang: string; text: string };

const UUID = '[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}';
const MENTION_RE = new RegExp(`^(@everyone|@here|@&\\{${UUID}\\}|@\\{${UUID}\\})`);

/** True when content contains angle-bracket HTML that must stay plain text (never executed). */
export function looksLikeRawHtml(content: string): boolean {
  return /<\/?[a-zA-Z][^>]*>/.test(content);
}

export function parseMessageMarkup(content: string): BlockNode[] {
  const blocks: BlockNode[] = [];
  const fence = /```([a-zA-Z0-9_+-]*)\n?([\s\S]*?)```/g;
  let last = 0;
  let match = fence.exec(content);
  while (match !== null) {
    if (match.index > last) {
      pushParagraphs(blocks, content.slice(last, match.index));
    }
    blocks.push({
      type: 'codeblock',
      lang: match[1] || 'text',
      text: (match[2] ?? '').replace(/\n$/, ''),
    });
    last = match.index + match[0].length;
    match = fence.exec(content);
  }
  if (last < content.length) {
    pushParagraphs(blocks, content.slice(last));
  }
  return blocks.length > 0 ? blocks : [{ type: 'paragraph', children: [] }];
}

function pushParagraphs(blocks: BlockNode[], chunk: string) {
  const parts = chunk.split(/\n{2,}/);
  for (const part of parts) {
    const trimmed = part.replace(/^\n+|\n+$/g, '');
    if (!trimmed) continue;
    blocks.push({
      type: 'paragraph',
      children: parseInline(trimmed.replace(/\n/g, ' ')),
    });
  }
}

function parseInline(input: string): InlineNode[] {
  const nodes: InlineNode[] = [];
  let i = 0;
  while (i < input.length) {
    if (input[i] === '\\' && i + 1 < input.length) {
      nodes.push({ type: 'text', text: input[i + 1]! });
      i += 2;
      continue;
    }

    const mention = MENTION_RE.exec(input.slice(i));
    if (mention) {
      const token = mention[1]!;
      if (token === '@everyone') nodes.push({ type: 'everyone' });
      else if (token === '@here') nodes.push({ type: 'here' });
      else if (token.startsWith('@&{')) {
        nodes.push({ type: 'role', id: token.slice(3, -1) });
      } else {
        nodes.push({ type: 'user', id: token.slice(2, -1) });
      }
      i += token.length;
      continue;
    }

    if (input[i] === '`') {
      const end = input.indexOf('`', i + 1);
      if (end !== -1) {
        nodes.push({ type: 'code', text: input.slice(i + 1, end) });
        i = end + 1;
        continue;
      }
    }

    if (input.startsWith('||', i)) {
      const end = input.indexOf('||', i + 2);
      if (end !== -1) {
        nodes.push({ type: 'spoiler', children: parseInline(input.slice(i + 2, end)) });
        i = end + 2;
        continue;
      }
    }

    if (input.startsWith('**', i)) {
      const end = input.indexOf('**', i + 2);
      if (end !== -1) {
        nodes.push({ type: 'bold', children: parseInline(input.slice(i + 2, end)) });
        i = end + 2;
        continue;
      }
    }

    if (input.startsWith('~~', i)) {
      const end = input.indexOf('~~', i + 2);
      if (end !== -1) {
        nodes.push({ type: 'strike', children: parseInline(input.slice(i + 2, end)) });
        i = end + 2;
        continue;
      }
    }

    if (input[i] === '*' || input[i] === '_') {
      const delim = input[i]!;
      const end = input.indexOf(delim, i + 1);
      if (end !== -1 && end > i + 1) {
        nodes.push({ type: 'italic', children: parseInline(input.slice(i + 1, end)) });
        i = end + 1;
        continue;
      }
    }

    const nextSpecial = findNextSpecial(input, i + 1);
    nodes.push({ type: 'text', text: input.slice(i, nextSpecial) });
    i = nextSpecial;
  }
  return mergeText(nodes);
}

function findNextSpecial(input: string, from: number): number {
  for (let j = from; j < input.length; j++) {
    const ch = input[j]!;
    if (ch === '\\' || ch === '`' || ch === '*' || ch === '_' || ch === '~' || ch === '|') {
      return j;
    }
    if (ch === '@') return j;
  }
  return input.length;
}

function mergeText(nodes: InlineNode[]): InlineNode[] {
  const out: InlineNode[] = [];
  for (const node of nodes) {
    const prev = out[out.length - 1];
    if (node.type === 'text' && prev?.type === 'text') {
      prev.text += node.text;
    } else {
      out.push(node);
    }
  }
  return out;
}

/** Flatten markup to plain text (for XSS assertions / excerpts). Never interprets HTML. */
export function markupToPlainText(nodes: BlockNode[]): string {
  const parts: string[] = [];
  for (const block of nodes) {
    if (block.type === 'codeblock') {
      parts.push(block.text);
      continue;
    }
    parts.push(inlineToPlain(block.children));
  }
  return parts.join('\n');
}

function inlineToPlain(nodes: InlineNode[]): string {
  return nodes
    .map((n) => {
      switch (n.type) {
        case 'text':
          return n.text;
        case 'code':
          return n.text;
        case 'everyone':
          return '@everyone';
        case 'here':
          return '@here';
        case 'user':
          return `@{${n.id}}`;
        case 'role':
          return `@&{${n.id}}`;
        case 'bold':
        case 'italic':
        case 'strike':
        case 'spoiler':
          return inlineToPlain(n.children);
        default:
          return '';
      }
    })
    .join('');
}
