/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_GATEWAY_DEBUG?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
