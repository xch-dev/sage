import { PasteInput } from '@/components/PasteInput';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Switch } from '@/components/ui/switch';
import { useBiometric } from '@/hooks/useBiometric';
import { useErrors } from '@/hooks/useErrors';
import { useScannerOrClipboard } from '@/hooks/useScannerOrClipboard';
import { useWalletConnect } from '@/hooks/useWalletConnect';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { platform } from '@tauri-apps/plugin-os';
import { TrashIcon } from 'lucide-react';
import { useEffect, useState } from 'react';
import { commands } from '../../bindings';
import { AgentSettings } from './AgentSettings';
import {
  ServerStatusIndicator,
  SettingItem,
  SettingsSection,
} from './SettingsSection';

export function ConnectionSettings() {
  const isMobile = platform() === 'ios' || platform() === 'android';

  return (
    <>
      <WalletConnectSettings />
      {!isMobile && <RpcSettings />}
      {!isMobile && <AgentSettings />}
    </>
  );
}

function WalletConnectSettings() {
  const { pair, sessions, disconnect, connecting } = useWalletConnect();
  const [uri, setUri] = useState<string>('');
  const [error, setError] = useState<string | null>(null);
  const { handleScanOrPaste, handleScanImage } = useScannerOrClipboard(
    (scanResValue) => {
      setUri(scanResValue);
    },
  );

  const handlePair = async () => {
    try {
      setError(null);
      await pair(uri);
      setUri('');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to connect');
    }
  };

  return (
    <SettingsSection title={t`WalletConnect`}>
      {sessions.length > 0 ? (
        sessions.map((session) => (
          <div
            key={session.topic}
            className='px-4 py-4 flex items-center justify-between gap-4'
          >
            <div className='flex gap-3 items-center'>
              <img
                src={session.peer?.metadata?.icons?.[0] ?? ''}
                alt={session.peer?.metadata?.name ?? t`Unknown App`}
                className='h-8 w-8 rounded-full'
              />
              <span className='font-medium'>
                {session.peer?.metadata?.name ?? t`Unknown App`}
              </span>
            </div>
            <Button
              variant='destructive'
              size='icon'
              onClick={() => disconnect(session.topic)}
            >
              <TrashIcon className='h-4 w-4' />
            </Button>
          </div>
        ))
      ) : (
        <div className='p-3 text-sm text-muted-foreground'>
          <Trans>No active sessions</Trans>
        </div>
      )}

      <div className='p-3'>
        <div className='flex flex-col gap-2'>
          <div className='flex gap-2 items-center'>
            <PasteInput
              value={uri}
              placeholder={t`WalletConnect URI`}
              onChange={(e) => setUri(e.target.value)}
              onEndIconClick={handleScanOrPaste}
              onScanImage={handleScanImage}
              disabled={connecting}
            />

            <Button onClick={handlePair} disabled={connecting} size='sm'>
              {connecting ? (
                <Trans>Connecting...</Trans>
              ) : (
                <Trans>Connect</Trans>
              )}
            </Button>
          </div>

          {error && <span className='text-sm text-destructive'>{error}</span>}
        </div>
      </div>
    </SettingsSection>
  );
}

function RpcSettings() {
  const { addError } = useErrors();
  const { promptIfEnabled } = useBiometric();

  const [isRunning, setIsRunning] = useState(false);
  const [runOnStartup, setRunOnStartup] = useState(false);

  useEffect(() => {
    // Get initial state
    Promise.all([commands.isRpcRunning(), commands.getRpcRunOnStartup()])
      .then(([running, startup]) => {
        setIsRunning(running);
        setRunOnStartup(startup);
      })
      .catch(addError);

    // Poll RPC status
    const interval = setInterval(() => {
      commands.isRpcRunning().then(setIsRunning).catch(addError);
    }, 1000);

    return () => clearInterval(interval);
  }, [addError]);

  const start = async () => {
    if (!(await promptIfEnabled())) return;

    commands
      .startRpcServer()
      .catch(addError)
      .then(() => setIsRunning(true));
  };

  const stop = () => {
    commands
      .stopRpcServer()
      .catch(addError)
      .then(() => setIsRunning(false));
  };

  const toggleRunOnStartup = async (checked: boolean) => {
    if (!(await promptIfEnabled())) return;

    commands
      .setRpcRunOnStartup(checked)
      .catch(addError)
      .then(() => setRunOnStartup(checked));
  };

  return (
    <SettingsSection title={t`RPC Server`}>
      <div className='p-3'>
        <div className='flex items-center justify-between'>
          <ServerStatusIndicator running={isRunning} />
          <Button
            variant={isRunning ? 'destructive' : 'default'}
            size='sm'
            onClick={isRunning ? stop : start}
          >
            {isRunning ? <Trans>Stop</Trans> : <Trans>Start</Trans>}
          </Button>
        </div>
      </div>

      <SettingItem
        label={t`Run on startup`}
        description={t`Automatically start the RPC server when the app launches`}
        control={
          <Switch checked={runOnStartup} onCheckedChange={toggleRunOnStartup} />
        }
      />

      <RpcPortSetting />
    </SettingsSection>
  );
}

function RpcPortSetting() {
  const { addError } = useErrors();

  const [savedPort, setSavedPort] = useState<number | null>(null);
  const [portText, setPortText] = useState('');

  useEffect(() => {
    commands
      .getRpcPort()
      .then((port) => {
        setSavedPort(port);
        setPortText(port.toString());
      })
      .catch(addError);
  }, [addError]);

  const port = Number(portText);
  const invalid = !/^\d+$/.test(portText) || port < 1 || port > 65535;

  const save = () => {
    if (invalid) {
      // An invalid port isn't saved, so the field goes back to the saved one.
      if (savedPort !== null) setPortText(savedPort.toString());
      return;
    }

    if (port === savedPort) return;

    commands
      .setRpcPort(port)
      .then(() => setSavedPort(port))
      .catch(addError);
  };

  return (
    <SettingItem
      label={t`Port`}
      description={t`The port the RPC server listens on. A change applies the next time the server starts.`}
      control={
        <Input
          type='number'
          className='w-[120px]'
          min={1}
          max={65535}
          value={portText}
          disabled={savedPort === null}
          onChange={(event) => setPortText(event.target.value)}
          onBlur={save}
        />
      }
    >
      {invalid && portText !== '' && (
        <p className='mt-2 text-sm text-destructive'>
          <Trans>The port must be between 1 and 65535</Trans>
        </p>
      )}
    </SettingItem>
  );
}
