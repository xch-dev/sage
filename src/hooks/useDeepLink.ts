import { useWallet } from '@/contexts/WalletContext';
import { isValidAddress } from '@/lib/utils';
import { useWalletState } from '@/state';
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

function parseDeepLinkUrl(url: string, prefix: string): DeepLinkData {
  if (!url.toLowerCase().startsWith(SCHEME_PREFIX)) {
    return null;
  }

  const payload = url.slice(SCHEME_PREFIX.length);

  const [mainPart, queryString] = payload.split('?');

  if (mainPart.startsWith('offer1')) {
    const result: OfferDeepLink = { type: 'offer', offerString: mainPart };

    if (queryString) {
      const params = new URLSearchParams(queryString);
      const fee = params.get('fee');
      if (fee) result.fee = fee;
    }

    return result;
  }

  if (isValidAddress(mainPart, prefix)) {
    const result: AddressDeepLink = {
      type: 'address',
      address: mainPart,
    };

    if (queryString) {
      const params = new URLSearchParams(queryString);
      const amount = params.get('amount');
      const fee = params.get('fee');
      const memo = params.get('memos');

      if (amount) result.amount = amount;
      if (fee) result.fee = fee;
      if (memo) result.memo = memo;
    }

    return result;
  }

  console.warn('Unrecognized deep link payload:', payload);
  return null;
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
  const walletState = useWalletState();
  const processedUrls = useRef<Set<string>>(new Set());

  useEffect(() => {
    let cleanup: (() => void) | null = null;

    const handleDeepLinkUrls = (urls: string[]) => {
      // Check if user is logged into a wallet
      if (!wallet) {
        toast.error(t`Please log into a wallet and try again`);
        return;
      }

      const prefix = walletState.sync.unit.ticker.toLowerCase();

      for (const url of urls) {
        if (processedUrls.current.has(url)) {
          continue;
        }
        processedUrls.current.add(url);

        const deepLinkData = parseDeepLinkUrl(url, prefix);
        if (!deepLinkData) {
          continue;
        }

        if (deepLinkData.type === 'offer') {
          let offerUrl = `/offers/view/${encodeURIComponent(deepLinkData.offerString)}`;
          if (deepLinkData.fee) {
            offerUrl += `?fee=${encodeURIComponent(deepLinkData.fee)}`;
          }
          navigate(offerUrl);
          break;
        }

        if (deepLinkData.type === 'address') {
          const params = new URLSearchParams();
          params.set('address', deepLinkData.address);
          if (deepLinkData.amount) params.set('amount', deepLinkData.amount);
          if (deepLinkData.fee) params.set('fee', deepLinkData.fee);
          if (deepLinkData.memo) params.set('memo', deepLinkData.memo);

          navigate(`/wallet/send/xch?${params.toString()}`);
          break;
        }
      }
    };

    const initDeepLink = async () => {
      try {
        const { getCurrent, onOpenUrl } = await import(
          '@tauri-apps/plugin-deep-link'
        );

        // Check if app was launched via deep link
        const initialUrls = await getCurrent();
        if (initialUrls && initialUrls.length > 0) {
          handleDeepLinkUrls(initialUrls);
        }

        // Listen for deep link events while the app is running
        const unlisten = await onOpenUrl(handleDeepLinkUrls);
        cleanup = unlisten;
      } catch (error) {
        // This can happen if the plugin isn't available on the current platform
        // or if there's a configuration issue. Log but don't crash.
        console.warn('Deep link handler not available:', error);
      }
    };

    initDeepLink();

    return () => {
      if (cleanup) {
        cleanup();
      }
    };
  }, [navigate, wallet, walletState]);
}
