/// <reference types="vite/client" />

// Allow importing JSON files as modules
declare module '*.json' {
  const value: any;
  export default value;
}
