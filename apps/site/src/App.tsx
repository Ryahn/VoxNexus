import { Navigate, Route, Routes } from 'react-router-dom';
import { DocPage } from '@/components/DocPage';
import { DocsShell } from '@/components/DocsShell';
import { getDocPage } from '@/lib/content';
import { ApiReferencePage } from '@/pages/ApiReferencePage';
import { DocsIndexPage } from '@/pages/DocsIndexPage';
import { LandingPage } from '@/pages/LandingPage';

function MarkdownDocRoute({ slug }: { slug: string }) {
  const page = getDocPage(slug);
  if (!page) {
    return <Navigate to="/docs" replace />;
  }
  return <DocPage title={page.title} body={page.body} />;
}

function DocsLayout() {
  return (
    <DocsShell>
      <Routes>
        <Route index element={<DocsIndexPage />} />
        <Route path="setup/compose" element={<MarkdownDocRoute slug="setup/compose" />} />
        <Route path="setup/config" element={<MarkdownDocRoute slug="setup/config" />} />
        <Route
          path="guides/api-conventions"
          element={<MarkdownDocRoute slug="guides/api-conventions" />}
        />
        <Route path="guides/auth" element={<MarkdownDocRoute slug="guides/auth" />} />
        <Route path="guides/codegen" element={<MarkdownDocRoute slug="guides/codegen" />} />
        <Route path="guides/gateway" element={<MarkdownDocRoute slug="guides/gateway" />} />
        <Route
          path="guides/observability"
          element={<MarkdownDocRoute slug="guides/observability" />}
        />
        <Route path="guides/storage" element={<MarkdownDocRoute slug="guides/storage" />} />
        <Route path="guides/search" element={<MarkdownDocRoute slug="guides/search" />} />
        <Route path="guides/jobs" element={<MarkdownDocRoute slug="guides/jobs" />} />
        <Route path="guides/database" element={<MarkdownDocRoute slug="guides/database" />} />
        <Route path="guides/ci" element={<MarkdownDocRoute slug="guides/ci" />} />
        <Route path="*" element={<Navigate to="/docs" replace />} />
      </Routes>
    </DocsShell>
  );
}

function ApiLayout() {
  return (
    <DocsShell fullWidth>
      <ApiReferencePage />
    </DocsShell>
  );
}

export function App() {
  return (
    <Routes>
      <Route path="/" element={<LandingPage />} />
      <Route path="/docs/api" element={<ApiLayout />} />
      <Route path="/docs/*" element={<DocsLayout />} />
      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  );
}
