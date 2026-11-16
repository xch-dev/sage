import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { useUpdates } from '@/contexts/UpdateContext';
import { useErrors } from '@/hooks/useErrors';
import { getConsoleLogs } from '@/lib/consoleCapture';
import { exportText, ExportType } from '@/lib/exportText';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { useVirtualizer } from '@tanstack/react-virtual';
import { openUrl } from '@tauri-apps/plugin-opener';
import { DownloadIcon } from 'lucide-react';
import { useCallback, useEffect, useRef, useState } from 'react';
import { commands, LogFile } from '../../bindings';
import { SettingItem, SettingsSection } from './SettingsSection';

const LOG_LEVELS = ['ERROR', 'WARN', 'INFO', 'DEBUG', 'TRACE'];

function LogLevelSettings() {
  const { addError } = useErrors();
  const [level, setLevel] = useState<string | null>(null);
  // The logger is set up when the app starts, so a change only applies after a restart.
  const [changed, setChanged] = useState(false);

  useEffect(() => {
    commands
      .getLogLevel()
      .then((level) => setLevel(level.toUpperCase()))
      .catch(addError);
  }, [addError]);

  return (
    <SettingsSection title={t`Logging`}>
      <SettingItem
        label={t`Log Level`}
        description={t`How much detail is written to the logs. More detail helps when reporting a problem.`}
        control={
          <Select
            value={level ?? ''}
            disabled={level === null}
            onValueChange={(value) => {
              commands
                .setLogLevel(value)
                .then(() => {
                  setLevel(value);
                  setChanged(true);
                })
                .catch(addError);
            }}
          >
            <SelectTrigger className='w-[140px]' aria-label={t`Log Level`}>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {LOG_LEVELS.map((option) => (
                <SelectItem key={option} value={option}>
                  {option}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        }
      >
        {changed && (
          <p className='mt-2 text-sm text-muted-foreground'>
            <Trans>Restart Sage for the new log level to apply.</Trans>
          </p>
        )}
      </SettingItem>
    </SettingsSection>
  );
}

function UpdateSettings() {
  const { enabled, setEnabled, status, update, checkNow } = useUpdates();
  const updateVersion = update?.version ?? '';

  const statusText = {
    idle: t`Automatic checks are off.`,
    checking: t`Checking for updates...`,
    'up-to-date': t`Sage is up to date.`,
    available: t`Sage ${updateVersion} is available.`,
    error: t`Couldn't reach GitHub to check for updates.`,
  }[status];

  return (
    <SettingsSection title={t`Updates`}>
      <SettingItem
        label={t`Check for Updates`}
        description={t`Ask GitHub for the latest release when Sage starts and every few hours.`}
        control={<Switch checked={enabled} onCheckedChange={setEnabled} />}
      />
      <SettingItem
        label={t`Latest Version`}
        description={statusText}
        control={
          update ? (
            <Button onClick={() => void openUrl(update.url)}>
              <Trans>View Release</Trans>
            </Button>
          ) : (
            <Button
              variant='outline'
              disabled={status === 'checking'}
              onClick={checkNow}
            >
              <Trans>Check Now</Trans>
            </Button>
          )
        }
      />
    </SettingsSection>
  );
}

export function AboutSettings() {
  const { supported, currentVersion } = useUpdates();

  return (
    <>
      <SettingsSection title={t`Application`}>
        <SettingItem
          label={t`Version`}
          control={
            <span className='text-sm text-muted-foreground'>
              {currentVersion}
            </span>
          }
        />
      </SettingsSection>
      {supported && <UpdateSettings />}
      <LogLevelSettings />
      <LogViewer />
    </>
  );
}

interface LogEntry {
  timestamp: string;
  level: string;
  message: string;
  source: 'backend' | 'frontend';
}

const LOG_LINE_REGEX =
  /^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z)\s+(\w+)\s+(.*)$/;

function LogViewer() {
  const { addError } = useErrors();

  const [logs, setLogs] = useState<LogFile[]>([]);
  const [logName, setLogName] = useState('');
  const [selectedLog, setSelectedLog] = useState<LogFile | null>(null);
  const parentRef = useRef<HTMLDivElement>(null);
  const [logLines, setLogLines] = useState<string[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedLevel, setSelectedLevel] = useState<string>('all');
  const [selectedSource, setSelectedSource] = useState<
    'all' | 'backend' | 'frontend'
  >('all');
  const [filteredEntries, setFilteredEntries] = useState<LogEntry[]>([]);
  const [isAtBottom, setIsAtBottom] = useState(true);

  const rowVirtualizer = useVirtualizer({
    count: filteredEntries.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 24,
    overscan: 5,
  });

  const scrollToBottom = useCallback(() => {
    if (!parentRef.current || !rowVirtualizer) return;
    const scrollHeight = rowVirtualizer.getTotalSize();
    parentRef.current.scrollTop = scrollHeight;
  }, [rowVirtualizer]);

  const checkIfAtBottom = useCallback(() => {
    if (!parentRef.current || !rowVirtualizer) return false;
    const { scrollTop, clientHeight } = parentRef.current;
    const scrollHeight = rowVirtualizer.getTotalSize();
    return scrollHeight - scrollTop - clientHeight < 10;
  }, [rowVirtualizer]);

  const handleScroll = useCallback(() => {
    setIsAtBottom(checkIfAtBottom());
  }, [checkIfAtBottom]);

  const virtualItems = rowVirtualizer.getVirtualItems();

  useEffect(() => {
    const items = virtualItems;
    if (
      items.length > 0 &&
      selectedLog &&
      (isAtBottom || parentRef.current?.scrollTop === undefined)
    ) {
      scrollToBottom();
    }
  }, [virtualItems, selectedLog, isAtBottom, scrollToBottom]);

  const updateLogs = useCallback(() => {
    commands
      .getLogs()
      .then((logs) =>
        setLogs(logs.sort((a, b) => b.name.localeCompare(a.name))),
      )
      .catch(addError);
  }, [addError]);

  useEffect(() => {
    updateLogs();

    const interval = setInterval(() => {
      updateLogs();
    }, 2000);

    return () => clearInterval(interval);
  }, [updateLogs]);

  useEffect(() => {
    if (logs.length > 0) {
      const defaultLog = logs[0];
      setLogName(defaultLog.name);
      setSelectedLog(defaultLog);
    }
  }, [logs]);

  useEffect(() => {
    if (selectedLog) {
      setLogLines(selectedLog.text.trim().split('\n'));
    } else {
      setLogLines([]);
    }
  }, [selectedLog]);

  useEffect(() => {
    const backendEntries: LogEntry[] = logLines.map((line) => {
      const match = line.match(LOG_LINE_REGEX);
      if (match) {
        return {
          timestamp: match[1],
          level: match[2],
          message: match[3],
          source: 'backend',
        };
      }
      return { timestamp: '', level: '', message: line, source: 'backend' };
    });

    const frontendEntries: LogEntry[] = getConsoleLogs().map((entry) => ({
      timestamp: entry.timestamp,
      level: entry.level,
      message: entry.message,
      source: 'frontend',
    }));

    const merged = [...backendEntries, ...frontendEntries].sort((a, b) => {
      if (!a.timestamp) return 1;
      if (!b.timestamp) return -1;
      return a.timestamp.localeCompare(b.timestamp);
    });

    const filtered = merged.filter((entry) => {
      if (
        searchQuery !== '' &&
        !`${entry.level} ${entry.message}`
          .toLowerCase()
          .includes(searchQuery.toLowerCase())
      ) {
        return false;
      }
      if (
        selectedLevel !== 'all' &&
        entry.level.toLowerCase() !== selectedLevel.toLowerCase()
      ) {
        return false;
      }
      if (selectedSource !== 'all' && entry.source !== selectedSource) {
        return false;
      }
      return true;
    });

    setFilteredEntries(filtered);
  }, [logLines, searchQuery, selectedLevel, selectedSource]);

  const handleLogChange = (name: string) => {
    setLogName(name);
    const log = logs.find((l) => l.name === name);
    setSelectedLog(log ?? null);
    setSearchQuery('');
    setSelectedLevel('all');
    setSelectedSource('all');
    setIsAtBottom(true);
  };

  const formatTimestamp = (timestamp: string) => {
    if (!timestamp) return '';
    try {
      return new Date(timestamp).toLocaleTimeString();
    } catch {
      return timestamp;
    }
  };

  const getLevelColor = (level: string) => {
    switch (level.toUpperCase()) {
      case 'ERROR':
        return 'text-red-500';
      case 'WARN':
        return 'text-yellow-600';
      case 'INFO':
        return 'text-blue-600';
      case 'DEBUG':
        return 'text-slate-500';
      default:
        return 'text-muted-foreground';
    }
  };

  const handleExport = () => {
    if (!selectedLog) return;

    const allBackend: LogEntry[] = logLines.map((line) => {
      const match = line.match(LOG_LINE_REGEX);
      if (match) {
        return {
          timestamp: match[1],
          level: match[2],
          message: match[3],
          source: 'backend',
        };
      }
      return { timestamp: '', level: '', message: line, source: 'backend' };
    });

    const allFrontend: LogEntry[] = getConsoleLogs().map((entry) => ({
      timestamp: entry.timestamp,
      level: entry.level,
      message: entry.message,
      source: 'frontend',
    }));

    const combined = [...allBackend, ...allFrontend]
      .sort((a, b) => {
        if (!a.timestamp) return 1;
        if (!b.timestamp) return -1;
        return a.timestamp.localeCompare(b.timestamp);
      })
      .map((entry) => {
        if (!entry.timestamp) return entry.message;
        const tag = entry.source === 'frontend' ? '[UI]' : '[BE]';
        return `${entry.timestamp} ${entry.level.padEnd(5)}  ${tag} ${entry.message}`;
      })
      .join('\n');

    exportText(combined, `${selectedLog.name}_combined`, ExportType.LOG);
  };

  return (
    <SettingsSection title={t`Log Viewer`}>
      <div className='p-3 space-y-4 max-w-full'>
        <div className='flex flex-col gap-4'>
          <div className='flex flex-col gap-2'>
            <Select value={logName} onValueChange={handleLogChange}>
              <SelectTrigger id='log' aria-label='Select file'>
                <SelectValue placeholder={<Trans>Select log file</Trans>} />
              </SelectTrigger>
              <SelectContent>
                {logs.map((log) => (
                  <SelectItem key={log.name} value={log.name}>
                    {log.name.replace('app.log.', '')}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            <div className='flex gap-2'>
              <Input
                type='text'
                placeholder={t`Search logs...`}
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
              />

              <Select value={selectedLevel} onValueChange={setSelectedLevel}>
                <SelectTrigger
                  id='level'
                  aria-label='Select log level'
                  className='w-[120px]'
                >
                  <SelectValue placeholder={<Trans>Log Level</Trans>} />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value='all'>
                    <Trans>All Levels</Trans>
                  </SelectItem>
                  <SelectItem value='error'>
                    <span className={getLevelColor('ERROR')}>ERROR</span>
                  </SelectItem>
                  <SelectItem value='warn'>
                    <span className={getLevelColor('WARN')}>WARN</span>
                  </SelectItem>
                  <SelectItem value='info'>
                    <span className={getLevelColor('INFO')}>INFO</span>
                  </SelectItem>
                  <SelectItem value='debug'>
                    <span className={getLevelColor('DEBUG')}>DEBUG</span>
                  </SelectItem>
                </SelectContent>
              </Select>

              <Select
                value={selectedSource}
                onValueChange={(v) =>
                  setSelectedSource(v as 'all' | 'backend' | 'frontend')
                }
              >
                <SelectTrigger
                  id='source'
                  aria-label='Select log source'
                  className='w-[130px]'
                >
                  <SelectValue placeholder={<Trans>Source</Trans>} />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value='all'>
                    <Trans>All Sources</Trans>
                  </SelectItem>
                  <SelectItem value='backend'>
                    <Trans>Backend</Trans>
                  </SelectItem>
                  <SelectItem value='frontend'>
                    <Trans>Frontend</Trans>
                  </SelectItem>
                </SelectContent>
              </Select>

              <Button variant='outline' onClick={handleExport}>
                <DownloadIcon className='h-4 w-4' />
              </Button>
            </div>
          </div>

          {selectedLog && filteredEntries.length === 0 && (
            <div className='text-center py-8 text-muted-foreground'>
              <Trans>No matching log entries found</Trans>
            </div>
          )}
        </div>

        {selectedLog && filteredEntries.length > 0 && (
          <div
            ref={parentRef}
            style={{ height: 'calc(100vh - 400px)', minHeight: '300px' }}
            className='border rounded-lg bg-muted/30 overflow-auto'
            onScroll={handleScroll}
          >
            <div
              style={{
                height: `${rowVirtualizer.getTotalSize()}px`,
                width: '100%',
                position: 'relative',
              }}
            >
              {rowVirtualizer.getVirtualItems().map((virtualRow) => {
                const entry = filteredEntries[virtualRow.index];
                const isFrontend = entry.source === 'frontend';

                return (
                  <div
                    key={virtualRow.index}
                    data-index={virtualRow.index}
                    ref={rowVirtualizer.measureElement}
                    className='absolute top-0 left-0 w-full hover:bg-muted/50 transition-colors'
                    style={{ transform: `translateY(${virtualRow.start}px)` }}
                  >
                    {entry.timestamp ? (
                      <div className='flex min-w-max px-2 py-0.5'>
                        <div className='w-[90px] whitespace-nowrap'>
                          <span className='text-xs text-muted-foreground font-mono'>
                            {formatTimestamp(entry.timestamp)}
                          </span>
                        </div>
                        <div className='w-[50px] whitespace-nowrap'>
                          <span
                            className={`text-xs font-medium ${getLevelColor(entry.level)}`}
                          >
                            {entry.level.padEnd(5, ' ')}
                          </span>
                        </div>
                        <div className='w-[28px] whitespace-nowrap'>
                          <span
                            className={`text-xs font-mono ${isFrontend ? 'text-purple-500' : 'text-muted-foreground/50'}`}
                          >
                            {isFrontend ? '[UI]' : '[BE]'}
                          </span>
                        </div>
                        <div className='flex-1 whitespace-nowrap'>
                          <span
                            className={`text-xs font-mono ${isFrontend ? 'text-purple-400' : ''}`}
                          >
                            {entry.message}
                          </span>
                        </div>
                      </div>
                    ) : (
                      <div className='px-2 py-0.5 whitespace-nowrap'>
                        <span className='text-xs font-mono'>
                          {entry.message}
                        </span>
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          </div>
        )}
      </div>
    </SettingsSection>
  );
}
