import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';

type DocPageProps = {
  title: string;
  body: string;
};

export function DocPage({ title, body }: DocPageProps) {
  return (
    <article className="prose-vox mx-auto max-w-3xl">
      <h1 className="mb-6 font-sans text-2xl font-semibold text-ink">{title}</h1>
      <ReactMarkdown remarkPlugins={[remarkGfm]}>{body}</ReactMarkdown>
    </article>
  );
}
