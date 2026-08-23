import { ApiReferenceReact } from '@scalar/api-reference-react';
import '@scalar/api-reference-react/style.css';

export function ApiReferencePage() {
  return (
    <div className="h-full min-h-0">
      <ApiReferenceReact
        configuration={{
          url: `${import.meta.env.BASE_URL}openapi.json`,
          theme: 'purple',
          hideModels: false,
        }}
      />
    </div>
  );
}
