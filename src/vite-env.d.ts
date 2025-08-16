/// <reference types="vite/client" />

// Allow importing JSON files as modules
declare module '*.json' {
  const value: never;
  export default value;
}
