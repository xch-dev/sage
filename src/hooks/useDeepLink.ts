import { useWallet } from '@/contexts/WalletContext';
import { isValidAddress } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { toast } from 'react-toastify';
import { commands } from '../bindings';

const SCHEME_PREFIX = 'sage:';

interface OfferDeepLink {
  type: 'offer';
  offerString: string;
  fee?: string;
}

interface AddressDeepLink {
  type: 'address';
  address: string;
  amount?: string;
  fee?: string;
  memo?: string;
  assetId?: string;
}

type DeepLinkData = OfferDeepLink | AddressDeepLink | null;

interface ParseResult {
  data: DeepLinkData;
  error?: string;
}

function decodeQueryString(queryString: string): URLSearchParams {
  let decoded = queryString;
  if (queryString.includes('%')) {
    try {
      decoded = decodeURIComponent(queryString);
    } catch {
      // If decoding fails, use the original string
    }
  }
  return new URLSearchParams(decoded);
}

function parseDeepLinkUrl(url: string): ParseResult {
  if (!url.toLowerCase().startsWith(SCHEME_PREFIX)) {
    return { data: null, error: 'invalid_scheme' };
  }

  const payload = url.slice(SCHEME_PREFIX.length);

  if (!payload) {
    return { data: null, error: 'empty_payload' };
  }

  const [mainPart, queryString] = payload.split('?');

  // Validate offer string: must start with offer1, be alphanumeric, and reasonable length
  // Chia offers are bech32m encoded, max ~10KB when compressed
  const MAX_OFFER_LENGTH = 15000;
  if (
    mainPart.startsWith('offer1') &&
    mainPart.length <= MAX_OFFER_LENGTH &&
    /^[a-z0-9]+$/.test(mainPart)
  ) {
    const result: OfferDeepLink = { type: 'offer', offerString: mainPart };

    if (queryString) {
      const params = decodeQueryString(queryString);
      const fee = params.get('fee');
      // Validate fee is a positive integer (mojos)
      if (fee && /^\d+$/.test(fee)) result.fee = fee;
    }

    return { data: result };
  }

  if (isValidAddress(mainPart, 'xch') || isValidAddress(mainPart, 'txch')) {
    const result: AddressDeepLink = {
      type: 'address',
      address: mainPart,
    };

    if (queryString) {
      const params = decodeQueryString(queryString);
      const amount = params.get('amount');
      const fee = params.get('fee');
      const memo = params.get('memos');
      const assetId = params.get('asset_id');

      // Validate amount and fee are positive integers (mojos)
      if (amount && /^\d+$/.test(amount)) result.amount = amount;
      if (fee && /^\d+$/.test(fee)) result.fee = fee;
      // Memo is freeform text but limit length to prevent abuse
      if (memo && memo.length <= 1000) result.memo = memo;
      // CAT asset IDs are 32-byte hex strings
      if (assetId && /^[0-9a-fA-F]{64}$/.test(assetId)) {
        result.assetId = assetId.toLowerCase();
      }
    }

    return { data: result };
  }

  console.warn('Unrecognized deep link payload:', payload);
  return { data: null, error: 'unrecognized_payload' };
}

/**
 * Hook to handle sage: deep links on all platforms.
 * Platform-specific notes:
 * - macOS: Deep links only work in the bundled app installed in /Applications.
 *          They will not work during development with `pnpm tauri dev`.
 * - Windows: Deep links are registered during app installation.
 * - Linux: Requires AppImage launcher for deep links to work, or use development mode
 *          with register_all() in Rust.
 * - iOS/Android: Deep links are configured via the mobile section in tauri.conf.json
 *                and work after the app is installed.
 */
// Cold-launch deep links are handled before WalletContext's async lookup
// (commands.getKey) has necessarily resolved, so wait briefly for it to
// settle rather than assuming "not logged in" the instant we see a null wallet.
const WALLET_INIT_POLL_MS = 50;
const WALLET_INIT_TIMEOUT_MS = 5000;

export function useDeepLink() {
  const navigate = useNavigate();
  const { wallet, isInitialized } = useWallet();

  // Use refs so the effect doesn't re-run when these change
  const walletRef = useRef(wallet);
  const isInitializedRef = useRef(isInitialized);
  const navigateRef = useRef(navigate);

  // Keep refs up to date
  useEffect(() => {
    walletRef.current = wallet;
    isInitializedRef.current = isInitialized;
    navigateRef.current = navigate;
  }, [wallet, isInitialized, navigate]);

  useEffect(() => {
    let cleanup: (() => void) | null = null;
    let isMounted = true;

    const waitForWalletInit = async () => {
      const deadline = Date.now() + WALLET_INIT_TIMEOUT_MS;
      while (isMounted && !isInitializedRef.current && Date.now() < deadline) {
        await new Promise((resolve) =>
          setTimeout(resolve, WALLET_INIT_POLL_MS),
        );
      }
    };

    const handleDeepLinkUrls = async (urls: string[]) => {
      for (const url of urls) {
        // Parse and validate URL first before checking wallet
        const { data: deepLinkData, error } = parseDeepLinkUrl(url);
        if (!deepLinkData) {
          if (error) {
            toast.error(t`Invalid deep link`);
          }
          continue;
        }

        // The initial (cold-launch) URL can arrive before the wallet lookup
        // has settled - wait for it so we don't report "not logged in"
        // for a wallet that's simply still loading.
        if (!isInitializedRef.current) {
          await waitForWalletInit();
        }

        if (!isMounted) return;

        // Only check wallet for valid deep links
        if (!walletRef.current) {
          toast.error(t`Please log into a wallet first`);
          return;
        }

        if (deepLinkData.type === 'offer') {
          let offerUrl = `/offers/view/${encodeURIComponent(deepLinkData.offerString)}`;
          if (deepLinkData.fee) {
            offerUrl += `?fee=${encodeURIComponent(deepLinkData.fee)}`;
          }
          navigateRef.current(offerUrl);
          break;
        }

        if (deepLinkData.type === 'address') {
          let assetPathSegment = 'xch';

          if (deepLinkData.assetId) {
            const assetId = deepLinkData.assetId;
            const { token } = await commands
              .getToken({ asset_id: assetId })
              .catch(() => ({ token: null }));

            if (!isMounted) return;

            if (!token) {
              toast.error(t`Unknown asset ${assetId}: it isn't in your wallet`);
              return;
            }

            assetPathSegment = assetId;
          }

          const params = new URLSearchParams();
          params.set('address', deepLinkData.address);
          if (deepLinkData.amount) params.set('amount', deepLinkData.amount);
          if (deepLinkData.fee) params.set('fee', deepLinkData.fee);
          if (deepLinkData.memo) params.set('memo', deepLinkData.memo);

          navigateRef.current(
            `/wallet/send/${assetPathSegment}?${params.toString()}`,
          );
          break;
        }
      }
    };

    const initDeepLink = async () => {
      try {
        const { getCurrent, onOpenUrl } =
          await import('@tauri-apps/plugin-deep-link');

        if (!isMounted) return;

        // Check if app was launched via deep link
        const initialUrls = await getCurrent();
        if (initialUrls && initialUrls.length > 0) {
          handleDeepLinkUrls(initialUrls);
        }

        if (!isMounted) return;

        // Listen for deep link events while the app is running
        // The single-instance plugin with "deep-link" feature automatically forwards URLs here
        cleanup = await onOpenUrl(handleDeepLinkUrls);
      } catch (error) {
        // This can happen if the plugin isn't available on the current platform
        // or if there's a configuration issue. Log but don't crash.
        console.warn('Deep link handler not available:', error);
      }
    };

    initDeepLink();

    return () => {
      isMounted = false;
      if (cleanup) {
        cleanup();
      }
    };
  }, []); // Empty deps - only run once
}
