import '@testing-library/jest-dom/vitest';
import { vi } from 'vitest';

// Mock @tauri-apps/api/core — invoke should throw by default to catch unmocked calls
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(() => {
    throw new Error('Unmocked Tauri invoke call');
  }),
  Channel: vi.fn(),
  transformCallback: vi.fn(),
}));

// Mock @tauri-apps/api/event
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async () => vi.fn()),
  once: vi.fn(async () => vi.fn()),
  emit: vi.fn(),
  TauriEvent: {},
}));

// Mock @tauri-apps/api/webviewWindow
vi.mock('@tauri-apps/api/webviewWindow', () => ({
  getCurrentWebviewWindow: vi.fn(() => ({
    listen: vi.fn(async () => vi.fn()),
    once: vi.fn(async () => vi.fn()),
    emit: vi.fn(),
  })),
  WebviewWindow: vi.fn(),
}));

// Mock @tauri-apps/api/window
vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn(() => ({
    listen: vi.fn(async () => vi.fn()),
    setTitle: vi.fn(),
    close: vi.fn(),
    minimize: vi.fn(),
    toggleMaximize: vi.fn(),
  })),
  Window: vi.fn(),
}));

// Mock Tauri plugins
vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({
  writeText: vi.fn(),
  readText: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-os', () => ({
  platform: vi.fn(async () => 'macos'),
  type: vi.fn(async () => 'Darwin'),
  arch: vi.fn(async () => 'aarch64'),
}));

vi.mock('@tauri-apps/plugin-opener', () => ({
  open: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
  save: vi.fn(),
  message: vi.fn(),
  ask: vi.fn(),
  confirm: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-fs', () => ({
  readFile: vi.fn(),
  writeFile: vi.fn(),
  readTextFile: vi.fn(),
  writeTextFile: vi.fn(),
  exists: vi.fn(),
  mkdir: vi.fn(),
  readDir: vi.fn(),
  remove: vi.fn(),
  rename: vi.fn(),
  BaseDirectory: {},
}));

vi.mock('@tauri-apps/plugin-biometric', () => ({
  authenticate: vi.fn(),
  checkStatus: vi.fn(),
  BiometricAuth: {},
}));

vi.mock('@tauri-apps/plugin-barcode-scanner', () => ({
  scan: vi.fn(),
  Format: {},
}));
