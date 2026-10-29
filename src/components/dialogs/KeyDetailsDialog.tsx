import { commands, KeyInfo, SecretKeyInfo, WalletInfo } from '@/bindings';
import { useBiometric } from '@/hooks/useBiometric';
import { useErrors } from '@/hooks/useErrors';
import { Trans } from '@lingui/react/macro';
import { useEffect, useState } from 'react';
import { Spoiler } from 'spoiled';
import { useTheme } from 'theme-o-rama';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '../ui/dialog';

export interface KeyDetailsDialogProps {
  keyInfo: KeyInfo | null;
  /** The wallet that uses this key, if any. */
  wallet: WalletInfo | null;
  onClose: () => void;
}

export function KeyDetailsDialog({
  keyInfo,
  wallet,
  onClose,
}: KeyDetailsDialogProps) {
  const { addError } = useErrors();
  const { promptIfEnabled } = useBiometric();
  const { currentTheme } = useTheme();
  const [secrets, setSecrets] = useState<SecretKeyInfo | null>(null);

  const fingerprint = keyInfo?.fingerprint ?? null;
  const hasSecrets = keyInfo?.has_secrets ?? false;

  useEffect(() => {
    setSecrets(null);

    if (fingerprint === null || !hasSecrets) return;

    let cancelled = false;

    (async () => {
      if (!(await promptIfEnabled())) return;

      commands
        .getSecretKey({ fingerprint })
        .then((data) => {
          if (!cancelled) setSecrets(data.secrets);
        })
        .catch(addError);
    })();

    return () => {
      cancelled = true;
    };
  }, [fingerprint, hasSecrets, promptIfEnabled, addError]);

  const spoilerTheme = currentTheme?.mostLike ?? 'light';

  return (
    <Dialog open={keyInfo !== null} onOpenChange={(open) => !open && onClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            <Trans>Key Details</Trans>
          </DialogTitle>
        </DialogHeader>
        {keyInfo && (
          <div className='space-y-4'>
            <div>
              <h3 className='font-semibold'>
                <Trans>Fingerprint</Trans>
              </h3>
              <p className='text-sm text-muted-foreground'>
                {keyInfo.fingerprint}
              </p>
            </div>
            <div>
              <h3 className='font-semibold'>
                <Trans>Wallet</Trans>
              </h3>
              <p className='text-sm text-muted-foreground'>
                {wallet ? wallet.name : <Trans>Not used by a wallet</Trans>}
              </p>
            </div>
            <div>
              <h3 className='font-semibold'>
                <Trans>Public Key</Trans>
              </h3>
              <p className='break-all text-sm text-muted-foreground'>
                {keyInfo.public_key}
              </p>
            </div>
            {!hasSecrets && (
              <div>
                <h3 className='font-semibold'>
                  <Trans>Secret Key</Trans>
                </h3>
                <p className='text-sm text-muted-foreground'>
                  <Trans>
                    Not stored. Only the public key is in the keychain.
                  </Trans>
                </p>
              </div>
            )}
            {secrets && (
              <>
                <div>
                  <h3 className='font-semibold'>
                    <Trans>Secret Key</Trans>
                  </h3>
                  <p className='break-all text-sm text-muted-foreground'>
                    <Spoiler theme={spoilerTheme}>{secrets.secret_key}</Spoiler>
                  </p>
                </div>
                {secrets.mnemonic && (
                  <div>
                    <h3 className='font-semibold'>
                      <Trans>Mnemonic</Trans>
                    </h3>
                    <p className='break-words text-sm text-muted-foreground'>
                      <Spoiler theme={spoilerTheme}>{secrets.mnemonic}</Spoiler>
                    </p>
                  </div>
                )}
              </>
            )}
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
