/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_DISABLE_OFFERS?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

// Allow importing JSON files as modules
declare module '*.json' {
  const value: never;
  export default value;
}
