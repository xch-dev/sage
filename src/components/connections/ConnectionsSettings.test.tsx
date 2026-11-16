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
import { afterEach, beforeAll, beforeEach, expect, it, vi } from 'vitest';
import type { ConnectionsView, ConnectionGrant, WalletInfo } from '@/bindings';
import { ConnectionsProvider } from '@/contexts/ConnectionsContext';
import { ConnectionsSettings } from './ConnectionsSettings';

const mocks = vi.hoisted(() => ({
  review: vi.fn(),
  create: vi.fn(),
  rotate: vi.fn(),
  revoke: vi.fn(),
  decide: vi.fn(),
  decideOperation: vi.fn(),
  server: vi.fn(),
  wallets: vi.fn(),
  copy: vi.fn(),
}));
vi.mock('@/bindings', () => ({
  commands: {
    connectionsReview: mocks.review,
    connectionsCreate: mocks.create,
    connectionsRotate: mocks.rotate,
    connectionsRevoke: mocks.revoke,
    connectionsDecidePairing: mocks.decide,
    connectionsDecideOperation: mocks.decideOperation,
    connectionsSetServer: mocks.server,
    getWallets: mocks.wallets,
  },
}));
vi.mock('@tauri-apps/plugin-os', () => ({ platform: () => 'macos' }));
vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({
  writeText: mocks.copy,
}));
vi.mock('theme-o-rama', () => ({ useTheme: () => ({ currentTheme: null }) }));
const wallets = [{ name: 'Savings', fingerprint: 7 }] as WalletInfo[];
const grant: ConnectionGrant = {
  name: 'Portfolio',
  kind: 'agent',
  capabilities: ['wallet.get_coins', 'wallet.get_version'],
  walletScope: { kind: 'selectedWallets', fingerprints: [7] },
};
let view: ConnectionsView;
beforeAll(() => {
  i18n.load('en', {});
  i18n.activate('en');
});
beforeEach(() => {
  vi.clearAllMocks();
  view = {
    enabled: false,
    running: false,
    port: 9258,
    certificatePath: '/test/scoped-rpc/server.crt',
    error: null,
    pending: [],
    operations: [],
    ownerOnlyMethods: ['delete_wallet', 'get_secret_key'],
    credentials: [],
    capabilities: [
      {
        key: 'wallet.get_coins',
        label: 'Read coins',
        description: 'Read wallet coins.',
        category: 'Coins',
        approval: false,
        methods: ['get_coins'],
      },
      {
        key: 'wallet.get_version',
        label: 'Read version',
        description: 'Read Sage version.',
        category: 'General',
        approval: false,
        methods: ['get_version'],
      },
    ],
  };
  mocks.review.mockImplementation(async () => structuredClone(view));
  mocks.wallets.mockResolvedValue({ wallets });
  mocks.copy.mockResolvedValue(undefined);
  mocks.create.mockImplementation(async (grant) => {
    const credential = { id: 'client-1', grant, createdAt: 1 };
    view.credentials.push(credential);
    return { credential, token: 'one-time-secret' };
  });
  mocks.decide.mockImplementation(async () => {
    view.pending = [];
  });
  mocks.decideOperation.mockImplementation(async () => {
    view.operations = [];
  });
  mocks.revoke.mockImplementation(async () => {
    view.credentials = [];
  });
  mocks.rotate.mockImplementation(async () => ({
    credential: view.credentials[0],
    token: 'replacement-secret',
  }));
  mocks.server.mockResolvedValue(undefined);
});
afterEach(cleanup);
function show(category = 'all') {
  render(
    <I18nProvider i18n={i18n}>
      <ConnectionsProvider>
        <ConnectionsSettings category={category} query='' wallets={wallets} />
      </ConnectionsProvider>
    </I18nProvider>,
  );
}
function request(expiresAt = Math.floor(Date.now() / 1000) + 300) {
  view.pending = [{ id: 'request-1', code: '1234ABCD', grant, expiresAt }];
}

it('issues only selected grants, shows the secret once and copies only on request', async () => {
  show();
  fireEvent.click(await screen.findByText('Create credential'));
  expect(
    (screen.getByRole('button', { name: 'Create' }) as HTMLButtonElement)
      .disabled,
  ).toBe(true);
  fireEvent.change(screen.getByLabelText('Connection name'), {
    target: { value: 'Reader' },
  });
  fireEvent.click(screen.getByLabelText('Savings (7)'));
  fireEvent.click(screen.getByLabelText(/Read coins/));
  fireEvent.click(screen.getByRole('button', { name: 'Create' }));
  await screen.findByText('Save your credential');
  expect(mocks.create).toHaveBeenCalledWith({
    name: 'Reader',
    kind: 'service',
    capabilities: ['wallet.get_coins'],
    walletScope: { kind: 'selectedWallets', fingerprints: [7] },
  });
  expect(
    (screen.getByLabelText('New credential') as HTMLTextAreaElement).value,
  ).toBe('one-time-secret');
  expect(mocks.copy).not.toHaveBeenCalled();
  fireEvent.click(screen.getByText('Copy credential'));
  await screen.findByText('Copied');
  expect(mocks.copy).toHaveBeenCalledWith('one-time-secret');
  fireEvent.click(screen.getByText('Done'));
  expect(screen.queryByLabelText('New credential')).toBeNull();
  expect(screen.getByText('Reader')).toBeTruthy();
});
it('requires explicit confirmation to replace or revoke an existing credential', async () => {
  view.credentials = [{ id: 'client-1', grant, createdAt: 1 }];
  show('agents');
  await screen.findByText('Portfolio');
  fireEvent.click(screen.getByText('Replace credential'));
  expect(mocks.rotate).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole('button', { name: 'Replace' }));
  await screen.findByText('Save your credential');
  expect(mocks.rotate).toHaveBeenCalledWith('client-1');
  fireEvent.click(screen.getByText('Done'));
  fireEvent.click(screen.getByText('Revoke access'));
  expect(mocks.revoke).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole('button', { name: 'Revoke' }));
  await waitFor(() => expect(screen.queryByText('Portfolio')).toBeNull());
  expect(mocks.revoke).toHaveBeenCalledWith('client-1');
});
it('requires code confirmation and allows approval of only a requested subset', async () => {
  request();
  show();
  await screen.findByText('Allow a new connection?');
  expect(
    (
      screen.getByRole('button', {
        name: 'Approve connection',
      }) as HTMLButtonElement
    ).disabled,
  ).toBe(true);
  expect(
    screen.queryByLabelText('All wallets, including wallets added later'),
  ).toBeNull();
  fireEvent.click(screen.getByLabelText(/Read version/));
  fireEvent.click(
    screen.getByLabelText('I started this request and the codes match.'),
  );
  await waitFor(() =>
    expect(
      (
        screen.getByRole('button', {
          name: 'Approve connection',
        }) as HTMLButtonElement
      ).disabled,
    ).toBe(false),
  );
  fireEvent.click(screen.getByText('Approve connection'));
  await waitFor(() =>
    expect(mocks.decide).toHaveBeenCalledWith('request-1', {
      ...grant,
      capabilities: ['wallet.get_coins'],
    }),
  );
  expect(mocks.create).not.toHaveBeenCalled();
});
it('closing defers a request and it can be reviewed again and rejected', async () => {
  request();
  show();
  await screen.findByText('Allow a new connection?');
  fireEvent.click(screen.getByText('Close'));
  expect(mocks.decide).not.toHaveBeenCalled();
  fireEvent.click(screen.getByText('Review request'));
  await screen.findByText('Allow a new connection?');
  fireEvent.click(screen.getByText('Reject'));
  await waitFor(() =>
    expect(mocks.decide).toHaveBeenCalledWith('request-1', null),
  );
});
it('does not approve expired requests', async () => {
  request(1);
  show();
  await screen.findByText('This request has expired.');
  expect(
    (
      screen.getByRole('button', {
        name: 'Approve connection',
      }) as HTMLButtonElement
    ).disabled,
  ).toBe(true);
  expect(mocks.decide).not.toHaveBeenCalled();
});
it('keeps failures visible and allows retry without issuing a second credential', async () => {
  view.credentials = [{ id: 'client-1', grant, createdAt: 1 }];
  mocks.rotate.mockRejectedValueOnce(new Error('Storage unavailable'));
  show();
  await screen.findByText('Portfolio');
  fireEvent.click(screen.getByText('Replace credential'));
  fireEvent.click(screen.getByRole('button', { name: 'Replace' }));
  await screen.findByText('Error: Storage unavailable');
  expect(screen.queryByLabelText('New credential')).toBeNull();
  fireEvent.click(screen.getByRole('button', { name: 'Replace' }));
  await screen.findByText('Save your credential');
  expect(mocks.rotate).toHaveBeenCalledTimes(2);
});
it('allows fixing a listener startup failure and rejects invalid ports', async () => {
  view.error = 'Address in use';
  show();
  await screen.findByText('Address in use');
  fireEvent.change(screen.getByLabelText('Port'), { target: { value: '80' } });
  fireEvent.click(screen.getByText('Enable listener'));
  await screen.findByText('Enter a port between 1024 and 65535.');
  expect(mocks.server).not.toHaveBeenCalled();
  fireEvent.change(screen.getByLabelText('Port'), {
    target: { value: '9260' },
  });
  fireEvent.click(screen.getByText('Enable listener'));
  await waitFor(() => expect(mocks.server).toHaveBeenCalledWith(true, 9260));
});

it('requires separate confirmation for an RPC operation and never treats a grant as approval', async () => {
  view.operations = [
    {
      id: 'op-1',
      credentialId: 'client-1',
      name: 'Portfolio',
      method: 'send_cat',
      parameters: '{"amount":"123","auto_submit":true}',
      fingerprint: 7,
      network: 'mainnet',
      expiresAt: Math.floor(Date.now() / 1000) + 300,
    },
  ];
  show();
  await screen.findByText('Approve RPC request?');
  expect(screen.getByLabelText('Request parameters').textContent).toContain(
    '"amount":"123"',
  );
  expect(
    (screen.getByRole('button', { name: 'Approve once' }) as HTMLButtonElement)
      .disabled,
  ).toBe(true);
  expect(mocks.decideOperation).not.toHaveBeenCalled();
  fireEvent.click(
    screen.getByLabelText('I reviewed this request and authorize it once.'),
  );
  fireEvent.click(screen.getByText('Approve once'));
  await waitFor(() =>
    expect(mocks.decideOperation).toHaveBeenCalledWith('op-1', true),
  );
  expect(mocks.decide).not.toHaveBeenCalled();
});
it('shows administration as owner-only and can search the expanded catalog', async () => {
  show();
  await screen.findByText('get_secret_key');
  fireEvent.click(screen.getByText('Create credential'));
  fireEvent.change(screen.getByLabelText('Search RPC permissions'), {
    target: { value: 'get_coins' },
  });
  expect(screen.getByLabelText(/Read coins/)).toBeTruthy();
  expect(screen.queryByLabelText(/Read version/)).toBeNull();
});
