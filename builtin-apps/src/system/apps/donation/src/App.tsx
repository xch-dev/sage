import { AppModalShell } from '@sage-app/ui';
import {
  formatSageError,
  useSageSystemClient,
  type DonationDetails,
} from '@sage-system-app/sdk';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { AmountPicker } from './components/AmountPicker';
import { DeveloperCard } from './components/DeveloperCard';
import { FeeInput } from './components/FeeInput';
import {
  appIconFromInline,
  DEFAULT_FEE_XCH,
  DEFAULT_USD,
  DEFAULT_XCH,
  getTargetAppId,
  parsePositiveNumber,
  type DonationMode,
  xchToMojos,
} from './utils';

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

  const canSend =
    !!details?.donationAddress &&
    !!derived.mojos &&
    feeMojos !== null &&
    !sending;

  async function sendDonation() {
    if (
      !canSend ||
      !details?.donationAddress ||
      !derived.mojos ||
      feeMojos === null
    ) {
      return;
    }

    setSending(true);
    setError(null);

    try {
      await sage.wallet.sendXch({
        address: details.donationAddress,
        amount: derived.mojos,
        fee: feeMojos,
        memos: [],
      });

      await sage.runtimeManager.closeSelf();
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

  return (
    <AppModalShell
      title='Support developer'
      appName={details.appName}
      appIcon={appIconFromInline(details.appIcon)}
      footer={
        <div className='flex justify-end gap-2'>
          <button
            type='button'
            disabled={sending}
            onClick={() => void sage.runtimeManager.closeSelf()}
            className='rounded-md border border-border px-3 py-1.5 text-sm hover:bg-muted disabled:opacity-50'
          >
            Cancel
          </button>

          <button
            type='button'
            disabled={!canSend}
            onClick={() => void sendDonation()}
            className='rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:opacity-90 disabled:opacity-50'
          >
            {sending ? 'Sending…' : 'Send support'}
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
