import { commands, SpendBundleJson, TransactionSummary } from '@/bindings';
import { useBiometric } from '@/hooks/useBiometric';
import { useErrors } from '@/hooks/useErrors';
import {
  aggregateSignatures,
  isUnsigned,
  parseSpendBundle,
  SpendBundleParseError,
  stringifySpendBundle,
} from '@/lib/spendBundle';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import {
  AlertCircleIcon,
  CheckCircleIcon,
  CopyIcon,
  PenLineIcon,
} from 'lucide-react';
import { useState } from 'react';
import { toast } from 'react-toastify';
import { AdvancedTransactionSummary } from '../AdvancedTransactionSummary';
import { ReadOnlyButton } from '../ReadOnlyButton';
import { Alert, AlertDescription, AlertTitle } from '../ui/alert';
import { Badge } from '../ui/badge';
import { Button } from '../ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../ui/dialog';
import { LoadingButton } from '../ui/loading-button';
import { Textarea } from '../ui/textarea';

export interface ImportSpendBundleDialogProps {
  open: boolean;
  setOpen: (open: boolean) => void;
  onSubmitted?: () => void;
}

export function ImportSpendBundleDialog({
  open,
  setOpen,
  onSubmitted,
}: ImportSpendBundleDialogProps) {
  const { addError } = useErrors();
  const { promptIfEnabled } = useBiometric();

  const [input, setInput] = useState('');
  const [parseError, setParseError] = useState<string | null>(null);
  const [spendBundle, setSpendBundle] = useState<SpendBundleJson | null>(null);
  const [summary, setSummary] = useState<TransactionSummary | null>(null);
  const [signedByWallet, setSignedByWallet] = useState(false);
  const [confirmUnsigned, setConfirmUnsigned] = useState(false);
  const [pending, setPending] = useState<'preview' | 'sign' | 'submit' | null>(
    null,
  );

  const unsigned = spendBundle
    ? isUnsigned(spendBundle.aggregated_signature)
    : false;

  const reset = () => {
    setInput('');
    setParseError(null);
    setSpendBundle(null);
    setSummary(null);
    setSignedByWallet(false);
    setConfirmUnsigned(false);
    setPending(null);
  };

  const close = () => {
    reset();
    setOpen(false);
  };

  const back = () => {
    setSpendBundle(null);
    setSummary(null);
    setSignedByWallet(false);
    setConfirmUnsigned(false);
  };

  const preview = async () => {
    let bundle: SpendBundleJson;

    try {
      bundle = parseSpendBundle(input);
    } catch (error) {
      if (error instanceof SpendBundleParseError) {
        setParseError(error.message);
        return;
      }
      throw error;
    }

    setParseError(null);
    setPending('preview');

    try {
      const data = await commands
        .viewCoinSpends({ coin_spends: bundle.coin_spends })
        .catch(addError);

      if (!data) return;

      setSpendBundle(bundle);
      setSummary(data.summary);
    } finally {
      setPending(null);
    }
  };

  const sign = async () => {
    if (!spendBundle || !(await promptIfEnabled())) return;

    setPending('sign');

    try {
      const data = await commands
        .signCoinSpends({
          coin_spends: spendBundle.coin_spends,
          partial: true,
        })
        .catch(addError);

      if (!data) return;

      const walletSignature = data.spend_bundle.aggregated_signature;

      if (isUnsigned(walletSignature)) {
        toast.info(t`This wallet has no signatures to add to this bundle`);
        return;
      }

      setSpendBundle({
        ...spendBundle,
        aggregated_signature: aggregateSignatures([
          spendBundle.aggregated_signature,
          walletSignature,
        ]),
      });
      setSignedByWallet(true);
      setConfirmUnsigned(false);
      toast.success(t`Spend bundle signed successfully`);
    } finally {
      setPending(null);
    }
  };

  const copy = () => {
    if (!spendBundle) return;

    writeText(stringifySpendBundle(spendBundle));
    toast.success(t`Spend bundle copied to clipboard`);
  };

  const submit = async () => {
    if (!spendBundle) return;

    if (unsigned && !confirmUnsigned) {
      setConfirmUnsigned(true);
      return;
    }

    setPending('submit');

    try {
      const data = await commands
        .submitTransaction({ spend_bundle: spendBundle })
        .catch(addError);

      if (!data) return;

      toast.success(t`Transaction submitted successfully`);
      onSubmitted?.();
      close();
    } finally {
      setPending(null);
    }
  };

  return (
    <Dialog open={open} onOpenChange={(open) => !open && close()}>
      <DialogContent className='max-w-xl max-h-[90vh] flex flex-col'>
        <DialogHeader>
          <DialogTitle>
            <Trans>Import Spend Bundle</Trans>
          </DialogTitle>
          <DialogDescription>
            {spendBundle ? (
              <Trans>
                Review the spend bundle below. You can add this wallet&apos;s
                signature, copy the result, or submit it to the network.
              </Trans>
            ) : (
              <Trans>
                Paste a JSON spend bundle with coin spends and an aggregated
                signature to review, sign, or submit it.
              </Trans>
            )}
          </DialogDescription>
        </DialogHeader>

        {spendBundle && summary ? (
          <div className='flex flex-col gap-3 overflow-y-auto min-h-0'>
            <div className='flex items-center gap-2 text-sm'>
              <Trans>Signature</Trans>
              {unsigned ? (
                <Badge variant='destructive'>
                  <Trans>Unsigned</Trans>
                </Badge>
              ) : (
                <Badge>
                  <Trans>Signed</Trans>
                </Badge>
              )}
            </div>

            <AdvancedTransactionSummary summary={summary} />

            {confirmUnsigned && (
              <Alert variant='warning'>
                <AlertCircleIcon className='h-4 w-4' aria-hidden='true' />
                <AlertTitle>
                  <Trans>Submit unsigned spend bundle?</Trans>
                </AlertTitle>
                <AlertDescription>
                  <Trans>
                    This spend bundle has no signature. It will be rejected
                    unless none of its coin spends require one.
                  </Trans>
                </AlertDescription>
              </Alert>
            )}
          </div>
        ) : (
          <div className='flex flex-col gap-3'>
            <Textarea
              value={input}
              onChange={(e) => {
                setInput(e.target.value);
                setParseError(null);
              }}
              placeholder={t`Paste spend bundle JSON`}
              aria-label={t`Spend bundle JSON`}
              className='min-h-[200px] font-mono text-xs'
              autoFocus
            />

            {parseError && (
              <Alert variant='destructive'>
                <AlertCircleIcon className='h-4 w-4' aria-hidden='true' />
                <AlertTitle>
                  <Trans>Invalid spend bundle</Trans>
                </AlertTitle>
                <AlertDescription className='break-all'>
                  {parseError}
                </AlertDescription>
              </Alert>
            )}
          </div>
        )}

        <DialogFooter className='gap-2 sm:gap-0 flex-wrap'>
          {spendBundle && summary ? (
            <>
              <Button
                variant='ghost'
                onClick={back}
                disabled={pending !== null}
              >
                <Trans>Back</Trans>
              </Button>
              <Button
                variant='outline'
                onClick={copy}
                disabled={pending !== null}
              >
                <CopyIcon className='h-4 w-4 mr-1' aria-hidden='true' />
                <Trans>Copy</Trans>
              </Button>
              <ReadOnlyButton
                variant='outline'
                requiresSigning
                onClick={sign}
                disabled={pending !== null || signedByWallet}
              >
                {signedByWallet ? (
                  <CheckCircleIcon
                    className='h-4 w-4 mr-1'
                    aria-hidden='true'
                  />
                ) : (
                  <PenLineIcon className='h-4 w-4 mr-1' aria-hidden='true' />
                )}
                {signedByWallet ? <Trans>Signed</Trans> : <Trans>Sign</Trans>}
              </ReadOnlyButton>
              <LoadingButton
                loading={pending === 'submit'}
                loadingText={t`Submitting`}
                onClick={submit}
                disabled={pending !== null}
                variant={confirmUnsigned ? 'destructive' : 'default'}
              >
                {confirmUnsigned ? (
                  <Trans>Submit Anyway</Trans>
                ) : (
                  <Trans>Submit</Trans>
                )}
              </LoadingButton>
            </>
          ) : (
            <>
              <Button variant='ghost' onClick={close}>
                <Trans>Cancel</Trans>
              </Button>
              <LoadingButton
                loading={pending === 'preview'}
                loadingText={t`Loading`}
                onClick={preview}
                disabled={!input.trim()}
              >
                <Trans>Review</Trans>
              </LoadingButton>
            </>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
