import { useWallet } from '@/contexts/WalletContext';
import { DeepLink, parseDeepLinkUrl } from '@/lib/deepLink';
import { addressInfo } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { useEffect, useState } from 'react';
import { NavigateFunction, useNavigate } from 'react-router-dom';
import { toast } from 'react-toastify';
import { commands } from '../bindings';

/** Handles `sage:` deep links; per-platform setup is in deep-links/README.md. */
export function useDeepLink() {
  const navigate = useNavigate();
  const { wallet, isInitialized } = useWallet();
  const [pendingLink, setPendingLink] = useState<DeepLink | null>(null);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let isMounted = true;

    // The first actionable link wins; unusable ones before it get a toast
    const queueFirstActionable = (urls: string[]) => {
      for (const url of urls) {
        const result = parseDeepLinkUrl(url);
        if (result.ok) {
          setPendingLink(result.link);
          return;
        }

        if (result.error === 'unsupported_action' && result.action) {
          const action = result.action;
          toast.error(t`Unsupported deep link action: ${action}`);
        } else {
          toast.error(t`Invalid deep link`);
        }
      }
    };

    (async () => {
      try {
        const { getCurrent, onOpenUrl } =
          await import('@tauri-apps/plugin-deep-link');

        // A link the app was launched with
        const initialUrls = await getCurrent();
        if (!isMounted) return;
        if (initialUrls) queueFirstActionable(initialUrls);

        // Links opened while running; the single-instance plugin forwards
        // these from a second launch on Windows and Linux
        const stop = await onOpenUrl(queueFirstActionable);
        if (isMounted) unlisten = stop;
        else stop();
      } catch (error) {
        // The plugin isn't available on every platform or configuration
        console.warn('Deep link handler not available:', error);
      }
    })();

    return () => {
      isMounted = false;
      unlisten?.();
    };
  }, []);

  // A cold-launch link can arrive before WalletContext's initial lookup has
  // settled, so hold it until then rather than reporting "not logged in"
  useEffect(() => {
    if (!pendingLink || !isInitialized) return;
    setPendingLink(null);

    if (!wallet) {
      toast.error(t`Please log into a wallet first`);
      return;
    }

    openDeepLink(pendingLink, navigate);
  }, [pendingLink, isInitialized, wallet, navigate]);
}

async function openDeepLink(link: DeepLink, navigate: NavigateFunction) {
  if (link.action === 'take_offer') {
    const query = link.fee
      ? `?${new URLSearchParams({ fee: link.fee }).toString()}`
      : '';
    navigate(`/offers/view/${encodeURIComponent(link.offer)}${query}`);
    return;
  }

  // The parser accepts any network's address; the wallet decides whether it
  // matches the active network, the same check the send form applies
  const isActiveNetwork = await commands
    .validateAddress(link.address)
    .catch(() => false);

  if (!isActiveNetwork) {
    const linkPrefix = addressInfo(link.address).prefix;
    const walletPrefix = await commands
      .getNetwork({})
      .then(({ network }) => network.prefix ?? network.ticker.toLowerCase())
      .catch(() => null);

    toast.error(
      walletPrefix
        ? t`This link is for a different network (${linkPrefix}); your wallet is on ${walletPrefix}`
        : t`This link is for a different network (${linkPrefix})`,
    );
    return;
  }

  if (link.assetId) {
    const assetId = link.assetId;
    const { token } = await commands
      .getToken({ asset_id: assetId })
      .catch(() => ({ token: null }));

    if (!token) {
      toast.error(t`Unknown asset ${assetId}: it isn't in your wallet`);
      return;
    }
  }

  const params = new URLSearchParams({ address: link.address });
  if (link.amount) params.set('amount', link.amount);
  if (link.fee) params.set('fee', link.fee);
  if (link.memos) params.set('memos', link.memos);

  navigate(`/wallet/send/${link.assetId ?? 'xch'}?${params.toString()}`);
}
