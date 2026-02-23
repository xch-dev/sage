import { useWallet } from '@/contexts/WalletContext';
import { isValidAddress } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { toast } from 'react-toastify';

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

      // Validate amount and fee are positive integers (mojos)
      if (amount && /^\d+$/.test(amount)) result.amount = amount;
      if (fee && /^\d+$/.test(fee)) result.fee = fee;
      // Memo is freeform text but limit length to prevent abuse
      if (memo && memo.length <= 1000) result.memo = memo;
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
export function useDeepLink() {
  const navigate = useNavigate();
  const { wallet } = useWallet();

  // Use refs so the effect doesn't re-run when these change
  const walletRef = useRef(wallet);
  const navigateRef = useRef(navigate);

  // Keep refs up to date
  useEffect(() => {
    walletRef.current = wallet;
    navigateRef.current = navigate;
  }, [wallet, navigate]);

  useEffect(() => {
    let cleanup: (() => void) | null = null;
    let isMounted = true;

    const handleDeepLinkUrls = (urls: string[]) => {
      for (const url of urls) {
        // Parse and validate URL first before checking wallet
        const { data: deepLinkData, error } = parseDeepLinkUrl(url);
        if (!deepLinkData) {
          if (error) {
            toast.error(t`Invalid deep link`);
          }
          continue;
        }

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
          const params = new URLSearchParams();
          params.set('address', deepLinkData.address);
          if (deepLinkData.amount) params.set('amount', deepLinkData.amount);
          if (deepLinkData.fee) params.set('fee', deepLinkData.fee);
          if (deepLinkData.memo) params.set('memo', deepLinkData.memo);

          navigateRef.current(`/wallet/send/xch?${params.toString()}`);
          break;
        }
      }
    };

    const initDeepLink = async () => {
      try {
        const { getCurrent, onOpenUrl } = await import(
          '@tauri-apps/plugin-deep-link'
        );

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
