// @vitest-environment jsdom

import { i18n } from '@lingui/core';
import { act, cleanup, renderHook, waitFor } from '@testing-library/react';
import {
  afterEach,
  beforeAll,
  beforeEach,
  describe,
  expect,
  it,
  vi,
} from 'vitest';

const XCH = 'xch1zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zygs3fazuv';
const TXCH = 'txch1zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zygsuw65al';
const ASSET_ID =
  'db1a9020d48d9d4ad22631b66ab4b9ebd3637ef7758ad38881348c5d24c38f20';
const OFFER_URL =
  'https://dexie.space/offers/6SEZH1GT5y1ecdaDKpSNCDVSDj4f8J2yjmsuBLyLmqYi';

const mocks = vi.hoisted(() => ({
  navigate: vi.fn(),
  toast: { error: vi.fn(), info: vi.fn() },
  wallet: { wallet: null as object | null, isInitialized: false },
  platform: { supportsSageApps: true },
  initialUrls: null as string[] | null,
  onOpenUrl: null as ((urls: string[]) => void) | null,
  commands: {
    validateAddress: vi.fn(),
    getNetwork: vi.fn(),
    getToken: vi.fn(),
    appsListRuntimes: vi.fn(),
    appsStartSystemApp: vi.fn(),
  },
}));

vi.mock('react-router-dom', async (importOriginal) => ({
  ...(await importOriginal<typeof import('react-router-dom')>()),
  useNavigate: () => mocks.navigate,
}));
vi.mock('react-toastify', () => ({ toast: mocks.toast }));
vi.mock('@/contexts/WalletContext', () => ({ useWallet: () => mocks.wallet }));
vi.mock('@/lib/apps/platform', () => ({
  get supportsSageApps() {
    return mocks.platform.supportsSageApps;
  },
}));
vi.mock('@/bindings', () => ({ commands: mocks.commands }));
vi.mock('@tauri-apps/plugin-deep-link', () => ({
  getCurrent: async () => mocks.initialUrls,
  onOpenUrl: async (handler: (urls: string[]) => void) => {
    mocks.onOpenUrl = handler;
    return () => {
      mocks.onOpenUrl = null;
    };
  },
}));

const { openInAppDeepLink, useDeepLink } = await import('./useDeepLink');

const MAINNET = { kind: 'mainnet', network: { prefix: 'xch', ticker: 'XCH' } };
const TESTNET = {
  kind: 'testnet',
  network: { prefix: 'txch', ticker: 'TXCH' },
};
const FROM_LINK = { state: { fromDeepLink: true } };

beforeAll(() => {
  i18n.loadAndActivate({ locale: 'en', messages: {} });
});

beforeEach(() => {
  vi.clearAllMocks();
  sessionStorage.clear();
  mocks.wallet = { wallet: {}, isInitialized: true };
  mocks.platform.supportsSageApps = true;
  mocks.initialUrls = null;
  mocks.commands.validateAddress.mockResolvedValue(true);
  mocks.commands.getNetwork.mockResolvedValue(MAINNET);
  mocks.commands.getToken.mockResolvedValue({ token: { asset_id: ASSET_ID } });
  mocks.commands.appsListRuntimes.mockResolvedValue([]);
  mocks.commands.appsStartSystemApp.mockResolvedValue({});
});

afterEach(cleanup);

// Mounts the hook and waits until it is listening for links
async function mount() {
  const hook = renderHook(() => useDeepLink());
  await waitFor(() => expect(mocks.onOpenUrl).not.toBeNull());
  return hook;
}

async function open(url: string) {
  await mount();
  await act(async () => mocks.onOpenUrl?.([url]));
}

describe('useDeepLink', () => {
  describe('wallet state', () => {
    it('holds a cold-launch link until the wallet lookup settles', async () => {
      mocks.wallet = { wallet: null, isInitialized: false };
      mocks.initialUrls = [`sage:send?address=${XCH}`];
      const hook = await mount();
      expect(mocks.navigate).not.toHaveBeenCalled();
      expect(mocks.toast.error).not.toHaveBeenCalled();

      mocks.wallet = { wallet: {}, isInitialized: true };
      hook.rerender();

      await waitFor(() =>
        expect(mocks.navigate).toHaveBeenCalledWith(
          `/wallet/send/xch?address=${XCH}`,
          FROM_LINK,
        ),
      );
    });

    it('does not replay the launch link when the hook remounts', async () => {
      mocks.initialUrls = [`sage:send?address=${XCH}`];
      const first = await mount();
      await waitFor(() => expect(mocks.navigate).toHaveBeenCalledTimes(1));
      first.unmount();

      await mount();
      await act(() => Promise.resolve());
      expect(mocks.navigate).toHaveBeenCalledTimes(1);
    });

    it('asks the user to log in when there is no wallet', async () => {
      mocks.wallet = { wallet: null, isInitialized: true };
      await open(`sage:send?address=${XCH}`);
      expect(mocks.toast.error).toHaveBeenCalledWith(
        'Please log into a wallet first',
      );
      expect(mocks.navigate).not.toHaveBeenCalled();
    });
  });

  describe('unusable links', () => {
    it('reports an invalid link', async () => {
      await open('sage:send');
      expect(mocks.toast.error).toHaveBeenCalledWith('Invalid deep link');
    });

    it('names an unsupported action', async () => {
      await open('sage:create_offer?x=1');
      expect(mocks.toast.error).toHaveBeenCalledWith(
        'Unsupported deep link action: create_offer',
      );
    });

    it('opens the first usable link after reporting the ones before it', async () => {
      await mount();
      await act(async () =>
        mocks.onOpenUrl?.(['sage:nope', `sage:send?address=${XCH}`]),
      );
      expect(mocks.toast.error).toHaveBeenCalledTimes(1);
      await waitFor(() => expect(mocks.navigate).toHaveBeenCalled());
    });
  });

  describe('send', () => {
    it('opens the send screen with the link details', async () => {
      await open(
        `sage:send?address=${XCH}&amount=10&fee=2&memos=hi&asset_id=${ASSET_ID}`,
      );
      await waitFor(() =>
        expect(mocks.navigate).toHaveBeenCalledWith(
          `/wallet/send/${ASSET_ID}?address=${XCH}&amount=10&fee=2&memos=hi`,
          FROM_LINK,
        ),
      );
    });

    it('rejects an address for another network, naming both', async () => {
      mocks.commands.validateAddress.mockResolvedValue(false);
      await open(`sage:send?address=${TXCH}`);
      await waitFor(() =>
        expect(mocks.toast.error).toHaveBeenCalledWith(
          'This link is for a different network (txch); your wallet is on xch',
        ),
      );
      expect(mocks.navigate).not.toHaveBeenCalled();
    });

    it('rejects an asset the wallet does not hold', async () => {
      mocks.commands.getToken.mockResolvedValue({ token: null });
      await open(`sage:send?address=${XCH}&asset_id=${ASSET_ID}`);
      await waitFor(() =>
        expect(mocks.toast.error).toHaveBeenCalledWith(
          `Unknown asset ${ASSET_ID}: it isn't in your wallet`,
        ),
      );
      expect(mocks.navigate).not.toHaveBeenCalled();
    });
  });

  describe('take_offer', () => {
    it('opens an offer page link on mainnet', async () => {
      await open(
        `sage:take_offer?url=${encodeURIComponent(OFFER_URL)}&fee=100`,
      );
      await waitFor(() =>
        expect(mocks.navigate).toHaveBeenCalledWith(
          `/offers/view/${encodeURIComponent(OFFER_URL)}?fee=100`,
          FROM_LINK,
        ),
      );
    });

    it('rejects an offer page link on testnet', async () => {
      mocks.commands.getNetwork.mockResolvedValue(TESTNET);
      await open(`sage:take_offer?url=${encodeURIComponent(OFFER_URL)}`);
      await waitFor(() =>
        expect(mocks.toast.error).toHaveBeenCalledWith(
          'This offer link is for mainnet; your wallet is on txch',
        ),
      );
      expect(mocks.navigate).not.toHaveBeenCalled();
    });
  });

  describe('install_app', () => {
    const LINK = 'sage:install_app?url=https://xch.place';

    it('opens the installer over the apps page with the URL prefilled', async () => {
      await open(LINK);
      await waitFor(() =>
        expect(mocks.commands.appsStartSystemApp).toHaveBeenCalledWith({
          kind: 'appInstall',
          source: { kind: 'selectSource', app_url: 'https://xch.place/' },
        }),
      );
      expect(mocks.navigate).toHaveBeenCalledWith('/apps');
    });

    it('leaves an open installer alone', async () => {
      mocks.commands.appsListRuntimes.mockResolvedValue([
        { app: { common: { identity: { id: 'app-install' } } } },
      ]);
      await open(LINK);
      await waitFor(() =>
        expect(mocks.toast.info).toHaveBeenCalledWith(
          'The app installer is already open',
        ),
      );
      expect(mocks.commands.appsStartSystemApp).not.toHaveBeenCalled();
    });

    it('reports that apps are unsupported on mobile', async () => {
      mocks.platform.supportsSageApps = false;
      await open(LINK);
      await waitFor(() =>
        expect(mocks.toast.error).toHaveBeenCalledWith(
          "Sage apps aren't supported on this device",
        ),
      );
      expect(mocks.navigate).not.toHaveBeenCalled();
    });
  });

  describe('openInAppDeepLink', () => {
    it('opens a scanned or pasted sage: link', async () => {
      await mount();
      let handled = false;
      act(() => {
        handled = openInAppDeepLink(` sage:send?address=${XCH} `);
      });
      expect(handled).toBe(true);
      await waitFor(() => expect(mocks.navigate).toHaveBeenCalled());
    });

    it('treats a malformed sage: link as handled and reports it', async () => {
      await mount();
      let handled = false;
      act(() => {
        handled = openInAppDeepLink('sage:send');
      });
      expect(handled).toBe(true);
      expect(mocks.toast.error).toHaveBeenCalledWith('Invalid deep link');
    });

    it('handles a link once when several scan fields receive it', async () => {
      await mount();
      act(() => {
        for (let i = 0; i < 4; i++) openInAppDeepLink('sage:send');
      });
      expect(mocks.toast.error).toHaveBeenCalledTimes(1);
    });

    it('leaves other text for the field', async () => {
      await mount();
      expect(openInAppDeepLink(XCH)).toBe(false);
    });

    it('does nothing once the hook is unmounted', async () => {
      const hook = await mount();
      hook.unmount();
      expect(openInAppDeepLink(`sage:send?address=${XCH}`)).toBe(false);
    });
  });
});
