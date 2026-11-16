// @vitest-environment jsdom
import { i18n } from '@lingui/core';
import { I18nProvider } from '@lingui/react';
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { afterEach, beforeAll, beforeEach, expect, it, vi } from 'vitest';
import type { AppPermissionReviews } from '@/bindings';
import { PermissionsSettings } from './PermissionsSettings';

const mocks = vi.hoisted(() => ({
  apps: vi.fn(),
  wallets: vi.fn(),
  rpc: vi.fn(),
  manage: vi.fn(),
}));
vi.mock('@/bindings', () => ({
  commands: {
    appsReviewPermissions: mocks.apps,
    getWallets: mocks.wallets,
    isRpcRunning: mocks.rpc,
  },
}));
vi.mock('@/lib/apps/openAppUpdate', () => ({
  openAppPermissionsReview: mocks.manage,
}));
vi.mock('@/hooks/useErrors', () => ({
  useErrors: () => ({ addError: vi.fn() }),
}));
vi.mock('@tauri-apps/plugin-os', () => ({ platform: () => 'macos' }));
vi.mock('@/hooks/useWalletConnect', () => ({
  useWalletConnect: () => ({
    sessions: [
      {
        topic: 'live',
        expiry: 4000000000,
        peer: {
          metadata: { name: 'Live exchange', url: 'https://exchange.example' },
        },
        namespaces: {
          chia: {
            methods: ['chia_getWallets'],
            accounts: ['chia:mainnet:7'],
            events: ['accountsChanged'],
          },
        },
      },
      {
        topic: 'expired',
        expiry: 1,
        peer: { metadata: { name: 'Expired exchange', url: '' } },
        namespaces: {},
      },
    ],
  }),
}));
const data: AppPermissionReviews = {
  apps: [
    {
      id: 'app-reader',
      name: 'Portfolio',
      walletScope: { kind: 'selectedWallets', fingerprints: [7] },
      capabilities: [
        {
          key: 'wallet.get_coins',
          label: 'Read coins',
          description: 'Read wallet coins.',
          wallet: true,
          automatic: false,
        },
        {
          key: 'app.get_info',
          label: 'Read app information',
          description: 'Read its identity.',
          wallet: false,
          automatic: true,
        },
      ],
      network: {
        whitelist: [{ scheme: 'https', host: 'example.com' }],
        whitelistByNetwork: {
          testnet11: [{ scheme: 'wss', host: 'test.example.com' }],
        },
      },
    },
  ],
  unavailableAppIds: [],
};

beforeAll(() => {
  i18n.load('en', {});
  i18n.activate('en');
});
beforeEach(() => {
  mocks.apps.mockReset().mockResolvedValue(data);
  mocks.wallets
    .mockReset()
    .mockResolvedValue({ wallets: [{ name: 'Savings', fingerprint: 7 }] });
  mocks.rpc.mockReset().mockResolvedValue(false);
  mocks.manage.mockReset().mockResolvedValue(undefined);
});
afterEach(cleanup);
function show() {
  render(
    <I18nProvider i18n={i18n}>
      <MemoryRouter>
        <PermissionsSettings />
      </MemoryRouter>
    </I18nProvider>,
  );
}
function tab(name: RegExp) {
  const element = screen.getByRole('tab', { name });
  fireEvent.mouseDown(element, { button: 0, ctrlKey: false });
  fireEvent.click(element);
}

it('shows effective app grants, network scopes, and owner RPC without inventing clients', async () => {
  show();
  await screen.findByText('Portfolio');
  expect(screen.getByText('Savings (7)')).toBeTruthy();
  expect(screen.getByText('Wallet permissions')).toBeTruthy();
  expect(screen.getByText('App permissions')).toBeTruthy();
  expect(screen.getByText('Included with app')).toBeTruthy();
  expect(screen.getByText('https://example.com')).toBeTruthy();
  expect(screen.getByText('testnet11')).toBeTruthy();
  expect(screen.getByText('Full-access RPC certificate')).toBeTruthy();
  expect(screen.getByText('Server stopped')).toBeTruthy();
  expect(screen.getByText('Live exchange')).toBeTruthy();
  expect(screen.queryByText('Expired exchange')).toBeNull();
  fireEvent.click(screen.getByText('Manage app permissions'));
  expect(mocks.manage).toHaveBeenCalledWith('app-reader');
});

it('filters categories and searches permission keys', async () => {
  show();
  await screen.findByText('Portfolio');
  tab(/^Apps/);
  expect(screen.queryByText('Full-access RPC certificate')).toBeNull();
  fireEvent.change(
    screen.getByRole('textbox', { name: 'Search permissions' }),
    { target: { value: 'wallet.get_coins' } },
  );
  expect(screen.getByText('Portfolio')).toBeTruthy();
  fireEvent.change(
    screen.getByRole('textbox', { name: 'Search permissions' }),
    { target: { value: 'no match' } },
  );
  expect(screen.getByText('No matching permissions')).toBeTruthy();
  tab(/^Agents/);
  expect(screen.getByText('No agent connections yet')).toBeTruthy();
});

it('reports unreadable app grants and supports retry without claiming there are none', async () => {
  mocks.apps.mockRejectedValueOnce(new Error('database unavailable'));
  show();
  await screen.findByText(
    'App permissions could not be loaded. Refresh to try again.',
  );
  tab(/^Apps/);
  expect(screen.queryByText('No connections to review')).toBeNull();
  fireEvent.click(screen.getByRole('button', { name: 'Refresh permissions' }));
  await screen.findByText('Portfolio');
  await waitFor(() =>
    expect(
      screen.queryByText(
        'App permissions could not be loaded. Refresh to try again.',
      ),
    ).toBeNull(),
  );
});
