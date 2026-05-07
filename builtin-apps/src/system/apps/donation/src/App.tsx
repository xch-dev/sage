import { AppModalShell, type AppIcon } from '@sage-app/ui';
import {
  formatSageError,
  useSageSystemClient,
  type DonationDetails,
  type SageAppIconView,
} from '@sage-system-app/sdk';
import { useEffect, useMemo, useState } from 'react';

type DonationMode = 'usd' | 'xch';

const DEFAULT_USD = '10';
const DEFAULT_XCH = '0.05';

function getTargetAppId() {
  return new URL(window.location.href).searchParams.get('appId');
}

function xchToMojos(xch: number): string {
  return String(Math.floor(xch * 1_000_000_000_000));
}

function parsePositiveNumber(value: string): number | null {
  const trimmed = value.trim();
  if (!trimmed) return null;

  const parsed = Number(trimmed);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : null;
}

function appIconFromInline(
  icon: SageAppIconView | null | undefined,
): AppIcon | null {
  if (!icon) return null;

  return {
    kind: 'bytes',
    icon: {
      bytes: icon.bytes,
      mime: icon.mime,
    },
  };
}

function inlineImageSrc(
  icon: SageAppIconView | null | undefined,
): string | null {
  if (!icon) return null;

  const bytes = new Uint8Array(icon.bytes);
  let binary = '';

  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }

  return `data:${icon.mime};base64,${btoa(binary)}`;
}

export function App() {
  const sage = useSageSystemClient();

  const [details, setDetails] = useState<DonationDetails | null>(null);
  const [mode, setMode] = useState<DonationMode>('usd');
  const [usdInput, setUsdInput] = useState(DEFAULT_USD);
  const [xchInput, setXchInput] = useState(DEFAULT_XCH);
  const [feeInput, setFeeInput] = useState('0');
  const [priceUsd, setPriceUsd] = useState<number | null>(null);
  const [priceLoading, setPriceLoading] = useState(true);
  const [sending, setSending] = useState(false);
  const [loaded, setLoaded] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const targetAppId = getTargetAppId();

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

    async function loadPrice() {
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

    void loadPrice();

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
      return { usd: null as number | null, xch, mojos: null as string | null };
    }

    return {
      usd: priceUsd ? xch * priceUsd : null,
      xch,
      mojos: xchToMojos(xch),
    };
  }, [mode, usdInput, xchInput, priceUsd]);

  const feeMojos = useMemo(() => {
    const feeXch = parsePositiveNumber(feeInput);

    if (feeInput.trim() === '' || feeInput.trim() === '0') {
      return '0';
    }

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

  const authorAvatarSrc = inlineImageSrc(details.authorAvatar);

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
            {sending ? 'Sending…' : 'Send donation'}
          </button>
        </div>
      }
    >
      <div className='space-y-4'>
        <div className='flex items-center gap-3 rounded-xl border bg-background/70 p-3'>
          {authorAvatarSrc ? (
            <img
              src={authorAvatarSrc}
              alt=''
              className='h-10 w-10 rounded-full border object-cover'
            />
          ) : (
            <div className='flex h-10 w-10 items-center justify-center rounded-full border bg-muted text-sm font-semibold'>
              {(details.authorName ?? details.appName)
                .slice(0, 1)
                .toUpperCase()}
            </div>
          )}

          <div className='min-w-0'>
            <div className='truncate text-sm font-semibold'>
              Support {details.authorName ?? details.appName}
            </div>
            <div className='truncate text-xs text-muted-foreground'>
              Developer of {details.appName}
            </div>
          </div>
        </div>

        <div>
          <div className='mb-2 flex items-center justify-between gap-3'>
            <div className='text-sm font-medium'>Amount</div>
            <div className='text-xs text-muted-foreground'>
              {priceLoading
                ? 'Loading XCH price…'
                : priceUsd !== null
                  ? `1 XCH ≈ $${priceUsd.toFixed(2)}`
                  : 'XCH price unavailable'}
            </div>
          </div>

          <div className='flex items-center gap-2'>
            <div className='flex items-center gap-1 rounded-lg border bg-background p-1'>
              <button
                type='button'
                disabled={priceUsd === null}
                onClick={() => setMode('usd')}
                className={[
                  'rounded-md px-3 py-1.5 text-sm font-medium',
                  mode === 'usd'
                    ? 'bg-primary text-primary-foreground'
                    : 'text-muted-foreground hover:bg-muted hover:text-foreground',
                  priceUsd === null ? 'cursor-not-allowed opacity-50' : '',
                ].join(' ')}
              >
                $
              </button>

              <button
                type='button'
                onClick={() => setMode('xch')}
                className={[
                  'rounded-md px-3 py-1.5 text-sm font-medium',
                  mode === 'xch'
                    ? 'bg-primary text-primary-foreground'
                    : 'text-muted-foreground hover:bg-muted hover:text-foreground',
                ].join(' ')}
              >
                XCH
              </button>
            </div>

            <div className='flex min-w-0 flex-1 items-center gap-2 rounded-lg border border-border px-3 py-2'>
              <input
                value={mode === 'usd' ? usdInput : xchInput}
                onChange={(event) => {
                  if (mode === 'usd') {
                    setUsdInput(event.target.value);
                  } else {
                    setXchInput(event.target.value);
                  }
                }}
                placeholder={mode === 'usd' ? '10.00' : '0.025'}
                inputMode='decimal'
                className='min-w-0 flex-1 bg-transparent text-sm outline-none'
              />
              <span className='text-xs font-medium text-muted-foreground'>
                {mode === 'usd' ? 'USD' : 'XCH'}
              </span>
            </div>
          </div>

          <div className='mt-1 text-right text-xs text-muted-foreground'>
            {mode === 'usd'
              ? derived.xch !== null
                ? `≈ ${derived.xch.toFixed(6)} XCH`
                : '—'
              : derived.usd !== null
                ? `≈ $${derived.usd.toFixed(2)}`
                : '—'}
          </div>
        </div>

        <label className='block'>
          <div className='mb-1 text-sm font-medium'>Fee</div>
          <div className='flex items-center gap-2 rounded-lg border border-border px-3 py-2'>
            <input
              value={feeInput}
              onChange={(event) => setFeeInput(event.target.value)}
              placeholder='0'
              inputMode='decimal'
              className='min-w-0 flex-1 bg-transparent text-sm outline-none'
            />
            <span className='text-xs font-medium text-muted-foreground'>
              XCH
            </span>
          </div>
        </label>

        <div className='rounded-lg border border-border bg-background/70 p-3'>
          <div className='text-xs font-medium uppercase tracking-wide text-muted-foreground'>
            Recipient
          </div>
          <div className='mt-1 break-all font-mono text-xs'>
            {details.donationAddress}
          </div>
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
