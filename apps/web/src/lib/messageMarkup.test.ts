import { describe, expect, it } from 'vitest';
import { looksLikeRawHtml, markupToPlainText, parseMessageMarkup } from './messageMarkup';

describe('messageMarkup', () => {
  it('renders bold, code, and spoiler tokens', () => {
    const blocks = parseMessageMarkup('**hi** `x` ||secret||');
    expect(blocks).toHaveLength(1);
    expect(blocks[0]).toMatchObject({ type: 'paragraph' });
    const kinds = (blocks[0] as { children: { type: string }[] }).children.map((c) => c.type);
    expect(kinds).toEqual(['bold', 'text', 'code', 'text', 'spoiler']);
  });

  it('keeps HTML payloads as plain text (XSS does not parse tags)', () => {
    const payload = `<script>alert('xss')</script><img src=x onerror=alert(1)>`;
    expect(looksLikeRawHtml(payload)).toBe(true);
    const blocks = parseMessageMarkup(payload);
    const plain = markupToPlainText(blocks);
    expect(plain).toBe(payload);
    // Only text / paragraph structure — no executable node types.
    for (const block of blocks) {
      expect(block.type).toBe('paragraph');
      if (block.type === 'paragraph') {
        expect(block.children.every((c) => c.type === 'text')).toBe(true);
      }
    }
  });

  it('does not treat javascript: URLs as links', () => {
    const blocks = parseMessageMarkup('[click](javascript:alert(1))');
    const plain = markupToPlainText(blocks);
    expect(plain).toContain('javascript:alert(1)');
    expect(JSON.stringify(blocks)).not.toContain('"type":"link"');
  });

  it('parses fenced code blocks', () => {
    const blocks = parseMessageMarkup('before\n```js\nconst x = 1;\n```\nafter');
    expect(blocks.some((b) => b.type === 'codeblock' && b.lang === 'js')).toBe(true);
  });
});
