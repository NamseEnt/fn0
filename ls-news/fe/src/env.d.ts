/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly PUBLIC_GITHUB_CLIENT_ID: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
