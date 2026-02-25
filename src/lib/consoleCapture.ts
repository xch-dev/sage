export interface ConsoleLogEntry {
  timestamp: string;
  level: 'LOG' | 'INFO' | 'WARN' | 'ERROR' | 'DEBUG';
  message: string;
}

const MAX_ENTRIES = 2000;
const entries: ConsoleLogEntry[] = [];

function serializeArgs(args: unknown[]): string {
  return args
    .map((a) => {
      if (typeof a === 'string') return a;
      try {
        return JSON.stringify(a);
      } catch {
        return String(a);
      }
    })
    .join(' ');
}

function capture(level: ConsoleLogEntry['level'], args: unknown[]) {
  entries.push({
    timestamp: new Date().toISOString(),
    level,
    message: serializeArgs(args),
  });
  if (entries.length > MAX_ENTRIES) {
    entries.splice(0, entries.length - MAX_ENTRIES);
  }
}

let installed = false;

export function installConsoleCapture() {
  if (installed) return;
  installed = true;

  const orig = {
    log: console.log.bind(console),
    info: console.info.bind(console),
    warn: console.warn.bind(console),
    error: console.error.bind(console),
    debug: console.debug.bind(console),
  };

  console.log = (...args: unknown[]) => {
    capture('LOG', args);
    orig.log(...args);
  };
  console.info = (...args: unknown[]) => {
    capture('INFO', args);
    orig.info(...args);
  };
  console.warn = (...args: unknown[]) => {
    capture('WARN', args);
    orig.warn(...args);
  };
  console.error = (...args: unknown[]) => {
    capture('ERROR', args);
    orig.error(...args);
  };
  console.debug = (...args: unknown[]) => {
    capture('DEBUG', args);
    orig.debug(...args);
  };
}

export function getConsoleLogs(): ConsoleLogEntry[] {
  return [...entries];
}

export function clearConsoleLogs() {
  entries.length = 0;
}
