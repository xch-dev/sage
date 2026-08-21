import { AppModalShell } from 'sage-app-ui';
import {
  formatSageError,
  useSageSystemClient,
  type DonationDetails,
} from 'sage-system-app-sdk';
import { CheckCircle2 } from 'lucide-react';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { AmountPicker } from './components/AmountPicker';
import { DeveloperBadge } from './components/DeveloperBadge';
import { DeveloperCard } from './components/DeveloperCard';
import { FeeInput } from './components/FeeInput';
import {
  appIconFromInline,
  createDonationReview,
  DEFAULT_FEE_XCH,
  DEFAULT_USD,
  DEFAULT_XCH,
  getTargetAppId,
  parsePositiveNumber,
  type DonationMode,
  type DonationReview,
  xchToMojos,
} from './utils';

type DonationStep = 'edit' | 'confirm' | 'complete';

export function App() {
  const sage = useSageSystemClient();

  const [details, setDetails] = useState<DonationDetails | null>(null);
  const [mode, setMode] = useState<DonationMode>('usd');
  const [usdInput, setUsdInput] = useState(DEFAULT_USD);
  const [xchInput, setXchInput] = useState(DEFAULT_XCH);
  const [feeInput, setFeeInput] = useState(DEFAULT_FEE_XCH);
  const [priceUsd, setPriceUsd] = useState<number | null>(null);
  const [priceLoading, setPriceLoading] = useState(true);
  const [sending, setSending] = useState(false);
  const [loaded, setLoaded] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [step, setStep] = useState<DonationStep>('edit');
  const [review, setReview] = useState<DonationReview | null>(null);

  const targetAppId = getTargetAppId();

  const loadPrice = useCallback(async () => {
    try {
      setPriceLoading(true);
      const price = await sage.wallet.getXchUsdPrice();

      setPriceUsd(price.usd);
    } catch {
      setPriceUsd(null);
      setMode('xch');
    } finally {
      setPriceLoading(false);
    }
  }, [sage]);

  useEffect(() => {
    let disposed = false;

    async function load() {
      try {
        if (!targetAppId) {
          throw new Error('Missing donation target app id.');
        }

        const next = await sage.donations.getDetails({ appId: targetAppId });

        if (!disposed) {
          setDetails(next);
        }
      } catch (err) {
        if (!disposed) {
          setError(formatSageError(err));
        }
      } finally {
        if (!disposed) {
          setLoaded(true);
        }
      }
    }

    void load();

    return () => {
      disposed = true;
    };
  }, [sage, targetAppId]);

  useEffect(() => {
    let disposed = false;

    async function load() {
      try {
        setPriceLoading(true);
        const price = await sage.wallet.getXchUsdPrice();

        if (!disposed) {
          setPriceUsd(price.usd);
        }
      } catch {
        if (!disposed) {
          setPriceUsd(null);
          setMode('xch');
        }
      } finally {
        if (!disposed) {
          setPriceLoading(false);
        }
      }
    }

    void load();

    return () => {
      disposed = true;
    };
  }, [sage]);

  const derived = useMemo(() => {
    if (mode === 'usd') {
      const usd = parsePositiveNumber(usdInput);

      if (!usd || !priceUsd) {
        return {
          usd,
          xch: null as number | null,
          mojos: null as string | null,
        };
      }

      const xch = usd / priceUsd;

      return {
        usd,
        xch,
        mojos: xchToMojos(xch),
      };
    }

    const xch = parsePositiveNumber(xchInput);

    if (!xch) {
      return {
        usd: null as number | null,
        xch,
        mojos: null as string | null,
      };
    }

    return {
      usd: priceUsd ? xch * priceUsd : null,
      xch,
      mojos: xchToMojos(xch),
    };
  }, [mode, usdInput, xchInput, priceUsd]);

  const feeMojos = useMemo(() => {
    const trimmed = feeInput.trim();

    if (trimmed === '' || trimmed === '0') {
      return '0';
    }

    const feeXch = parsePositiveNumber(trimmed);
    return feeXch ? xchToMojos(feeXch) : null;
  }, [feeInput]);

  const canReview =
    !!details?.donationAddress && !!derived.mojos && feeMojos !== null;

  function reviewDonation() {
    if (
      !canReview ||
      !details?.donationAddress ||
      !derived.mojos ||
      feeMojos === null
    ) {
      return;
    }

    setError(null);
    setReview(
      createDonationReview(
        details.donationAddress,
        derived.mojos,
        feeMojos,
        derived.usd,
      ),
    );
    setStep('confirm');
  }

  function editDonation() {
    if (sending) return;

    setError(null);
    setReview(null);
    setStep('edit');
  }

  async function sendDonation() {
    if (sending || !review) {
      return;
    }

    setSending(true);
    setError(null);

    try {
      await sage.wallet.sendXch({
        address: review.donationAddress,
        amount: review.amountMojos,
        fee: review.feeMojos,
        memos: [],
      });

      setStep('complete');
    } catch (err) {
      setError(formatSageError(err));
    } finally {
      setSending(false);
    }
  }

  if (!loaded) {
    return (
      <AppModalShell
        title='Support developer'
        appName='Donation'
        appIcon={null}
      >
        <div className='text-sm text-muted-foreground'>Loading donation…</div>
      </AppModalShell>
    );
  }

  if (!details) {
    return (
      <AppModalShell
        title='Donation unavailable'
        appName='Donation'
        appIcon={null}
        footer={
          <div className='flex justify-end'>
            <button
              type='button'
              onClick={() => void sage.runtimeManager.closeSelf()}
              className='rounded-md border border-border px-3 py-1.5 text-sm hover:bg-muted'
            >
              Close
            </button>
          </div>
        }
      >
        <div className='text-sm text-muted-foreground'>
          {error ?? 'This app does not have a donation address.'}
        </div>
      </AppModalShell>
    );
  }

  const appIcon = appIconFromInline(details.appIcon);

  if (step === 'confirm' && review) {
    return (
      <AppModalShell
        title='Confirm support'
        appName={details.appName}
        appIcon={appIcon}
        footer={
          <div className='flex justify-end gap-2'>
            <button
              type='button'
              disabled={sending}
              onClick={editDonation}
              className='rounded-md border border-border px-3 py-1.5 text-sm hover:bg-muted disabled:opacity-50'
            >
              Back
            </button>

            <button
              type='button'
              disabled={sending}
              onClick={() => void sendDonation()}
              className='rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:opacity-90 disabled:opacity-50'
            >
              {sending ? 'Sending…' : 'Confirm'}
            </button>
          </div>
        }
      >
        <div className='space-y-5 py-5 text-center'>
          <div className='flex flex-col items-center'>
            <p className='text-sm text-muted-foreground'>
              You are about to send
            </p>

            <div className='mt-2 text-2xl font-semibold'>
              {review.amountXch} XCH
              {review.approximateUsd !== null ? (
                <span className='ml-2 text-sm font-normal text-muted-foreground'>
                  (~${review.approximateUsd.toFixed(2)})
                </span>
              ) : null}
            </div>

            <div className='mt-4 flex items-center gap-2'>
              <span className='text-sm text-muted-foreground'>to</span>
              <DeveloperBadge details={details} />
            </div>

            <p className='mt-4 text-sm text-muted-foreground'>
              with network fee {review.feeXch} XCH
            </p>
          </div>

          {error ? (
            <div className='rounded-lg border border-destructive/40 bg-destructive/10 p-2 text-sm text-destructive'>
              {error}
            </div>
          ) : null}
        </div>
      </AppModalShell>
    );
  }

  if (step === 'complete' && review) {
    const developerName = details.authorName ?? details.appName;

    return (
      <AppModalShell
        title='Thank you'
        appName={details.appName}
        appIcon={appIcon}
        footer={
          <div className='flex justify-end'>
            <button
              type='button'
              onClick={() => void sage.runtimeManager.closeSelf()}
              className='rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:opacity-90'
            >
              Close
            </button>
          </div>
        }
      >
        <div className='flex flex-col items-center py-5 text-center'>
          <div className='flex h-14 w-14 items-center justify-center rounded-full bg-primary/10 text-primary'>
            <CheckCircle2 className='h-8 w-8' aria-hidden='true' />
          </div>
          <h2 className='mt-4 text-lg font-semibold'>
            {developerName} thanks you!
          </h2>
          <p className='mt-2 max-w-sm text-sm text-muted-foreground'>
            Your support of {review.amountXch} XCH was sent successfully and
            helps the developer continue improving {details.appName}.
          </p>
        </div>
      </AppModalShell>
    );
  }

  return (
    <AppModalShell
      title='Support developer'
      appName={details.appName}
      appIcon={appIcon}
      footer={
        <div className='flex justify-end gap-2'>
          <button
            type='button'
            onClick={() => void sage.runtimeManager.closeSelf()}
            className='rounded-md border border-border px-3 py-1.5 text-sm hover:bg-muted'
          >
            Cancel
          </button>

          <button
            type='button'
            disabled={!canReview}
            onClick={reviewDonation}
            className='rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:opacity-90 disabled:opacity-50'
          >
            Send support
          </button>
        </div>
      }
    >
      <div className='space-y-4'>
        <DeveloperCard details={details} />

        <AmountPicker
          mode={mode}
          setMode={setMode}
          usdInput={usdInput}
          setUsdInput={setUsdInput}
          xchInput={xchInput}
          setXchInput={setXchInput}
          priceUsd={priceUsd}
          priceLoading={priceLoading}
          onRefreshPrice={() => void loadPrice()}
          derivedXch={derived.xch}
          derivedUsd={derived.usd}
        />

        <FeeInput value={feeInput} onChange={setFeeInput} />

        {error ? (
          <div className='rounded-lg border border-destructive/40 bg-destructive/10 p-2 text-sm text-destructive'>
            {error}
          </div>
        ) : null}
      </div>
    </AppModalShell>
  );
}
