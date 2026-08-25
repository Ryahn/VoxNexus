# Message formatting

Messages store **plain text** with a Discord-like markup subset. The server does not render HTML. Clients parse and render safely (React text nodes — no `dangerouslySetInnerHTML`), so raw HTML and `javascript:` URLs never execute.

## Allowed markup

| Syntax | Effect |
|---|---|
| `**bold**` | Bold |
| `*italic*` or `_italic_` | Italic |
| `~~strike~~` | Strikethrough |
| `` `code` `` | Inline code |
| Fenced \`\`\`lang … \`\`\` | Code block (optional language tag) |
| `\|\|spoiler\|\|` | Spoiler (hidden until click) |
| `@{uuid}` / `@&{uuid}` / `@everyone` / `@here` | Mentions (see [Channels](/docs/guides/channels)) |
| `\*` `\_` `\`` etc. | Escape the next character |

Blank lines split paragraphs. Unsupported syntax (including raw HTML tags and Markdown links) is shown as literal text.

## Security

- Angle-bracket HTML (`<script>`, `<img onerror=…>`) is **not** parsed as markup.
- There is no link/image auto-render in this subset (richer HTML/Markdown lives in a later docs/forum path).
- Spoilers are client UI only; content is still in the stored string.

## UI

The live channel composer has a **Preview** control to render draft markup before send. Click a spoiler in chat to reveal it.
