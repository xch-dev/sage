import { commands } from '@/bindings';
import SafeAreaView from '@/components/SafeAreaView';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { CustomError } from '@/contexts/ErrorContext';
import { useErrors } from '@/hooks/useErrors';
import { toCustomError } from '@/lib/errors';
import { Trans } from '@lingui/react/macro';
import { platform } from '@tauri-apps/plugin-os';
import { AlertTriangleIcon } from 'lucide-react';

export interface StartupErrorProps {
  error: CustomError;
}

// Shown instead of the app when the backend fails to initialize, since the wallet
// can't be used safely without its keychain and config being loaded.
export function StartupError({ error }: StartupErrorProps) {
  const { addError } = useErrors();
  const alreadyRunning = error.kind === 'already_running';

  // Mobile apps are expected to be closed by the user rather than exit themselves
  const currentPlatform = platform();
  const isMobile = currentPlatform === 'ios' || currentPlatform === 'android';

  return (
    <SafeAreaView>
      <div className='flex flex-1 items-center justify-center p-4'>
        <Card className='w-full max-w-md'>
          <CardHeader>
            <CardTitle className='flex items-center gap-2'>
              <AlertTriangleIcon className='h-5 w-5 text-destructive' />
              {alreadyRunning ? (
                <Trans>Sage is already running</Trans>
              ) : (
                <Trans>Sage could not start</Trans>
              )}
            </CardTitle>
            <CardDescription>
              {alreadyRunning ? (
                <Trans>
                  Another instance of Sage is already using this wallet data.
                  Close this one and use the other instance instead.
                </Trans>
              ) : (
                <Trans>
                  An error occurred while loading your wallet data. Resolve the
                  problem below, then reopen Sage.
                </Trans>
              )}
            </CardDescription>
          </CardHeader>
          {!alreadyRunning && (
            <CardContent>
              <p className='text-sm whitespace-pre-wrap break-words hyphens-auto'>
                {error.reason}
              </p>
            </CardContent>
          )}
          {!isMobile && (
            <CardFooter className='justify-end'>
              <Button
                onClick={() =>
                  commands.quit().catch((e) => addError(toCustomError(e)))
                }
              >
                <Trans>Quit Sage</Trans>
              </Button>
            </CardFooter>
          )}
        </Card>
      </div>
    </SafeAreaView>
  );
}
