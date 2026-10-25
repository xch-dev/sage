import { useWallet } from '@/contexts/WalletContext';
import { DeepLink, parseDeepLinkUrl } from '@/lib/deepLink';
import { addressInfo } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { toast } from 'react-toastify';
import { commands } from '../bindings';

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

    const handleDeepLink = async (deepLinkData: DeepLink) => {
      if (!isInitializedRef.current) {
        await waitForWalletInit();
      }

      if (!isMounted) return;

      // Only check wallet for valid deep links
      if (!walletRef.current) {
        toast.error(t`Please log into a wallet first`);
        return;
      }

      if (deepLinkData.action === 'take_offer') {
        let offerUrl = `/offers/view/${encodeURIComponent(deepLinkData.offer)}`;
        if (deepLinkData.fee) {
          offerUrl += `?fee=${encodeURIComponent(deepLinkData.fee)}`;
        }
        navigateRef.current(offerUrl);
        return;
      }

      // The parser accepts any network's address; the wallet decides whether it
      // matches the active network, the same check the send form applies
      const isActiveNetwork = await commands
        .validateAddress(deepLinkData.address)
        .catch(() => false);

      if (!isMounted) return;

      if (!isActiveNetwork) {
        const linkPrefix = addressInfo(deepLinkData.address).prefix;
        const walletPrefix = await commands
          .getNetwork({})
          .then(({ network }) => network.prefix ?? network.ticker.toLowerCase())
          .catch(() => null);

        if (!isMounted) return;

        toast.error(
          walletPrefix
            ? t`This link is for a different network (${linkPrefix}); your wallet is on ${walletPrefix}`
            : t`This link is for a different network (${linkPrefix})`,
        );
        return;
      }

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
      if (deepLinkData.memos) params.set('memos', deepLinkData.memos);

      navigateRef.current(
        `/wallet/send/${assetPathSegment}?${params.toString()}`,
      );
    };

    const handleDeepLinkUrls = async (urls: string[]) => {
      for (const url of urls) {
        // Parse and validate URL first before checking wallet
        const result = parseDeepLinkUrl(url);
        if (!result.ok) {
          if (result.error === 'unsupported_action' && result.action) {
            const action = result.action;
            toast.error(t`Unsupported deep link action: ${action}`);
          } else {
            toast.error(t`Invalid deep link`);
          }
          // Skip past unusable links to see if a later one is actionable
          continue;
        }

        // The first actionable link wins - stop regardless of how it turned out
        await handleDeepLink(result.link);
        return;
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
