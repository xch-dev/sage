import { useWallet } from '@/contexts/WalletContext';
import { t } from '@lingui/core/macro';
import { useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { toast } from 'react-toastify';

const SCHEME_PREFIX = 'sage:';

function parseDeepLinkUrl(url: string): string | null {
  if (!url.toLowerCase().startsWith(SCHEME_PREFIX)) {
    return null;
  }

  const offerString = url.slice(SCHEME_PREFIX.length);

  // Basic validation - offer strings should start with 'offer1'
  if (!offerString || !offerString.startsWith('offer1')) {
    console.warn('Invalid offer string in deep link:', offerString);
    return null;
  }

  return offerString;
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
  const processedUrls = useRef<Set<string>>(new Set());

  useEffect(() => {
    let cleanup: (() => void) | null = null;

    const handleDeepLinkUrls = (urls: string[]) => {
      for (const url of urls) {
        if (processedUrls.current.has(url)) {
          continue;
        }
        processedUrls.current.add(url);

        const offerString = parseDeepLinkUrl(url);
        if (offerString) {
          // Check if user is logged into a wallet
          if (!wallet) {
            toast.error(t`Please log into a wallet and try again`);
            break;
          }
          // Navigate to the offer view page
          navigate(`/offers/view/${encodeURIComponent(offerString)}`);
          break;
        }
      }
    };

    const initDeepLink = async () => {
      try {
        // Dynamically import the deep-link plugin
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
  }, [navigate, wallet]);
}
