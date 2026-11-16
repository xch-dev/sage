// @vitest-environment jsdom

import { i18n } from '@lingui/core';
import { I18nProvider } from '@lingui/react';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import {
  afterEach,
  beforeAll,
  beforeEach,
  describe,
  expect,
  it,
  vi,
} from 'vitest';
import {
  MemoryRouter,
  Route,
  Routes,
  useLocation,
  useNavigate,
} from 'react-router-dom';
import Settings from './Settings';

let wallet: { name: string; fingerprint: number } | null = null;

vi.mock('@/contexts/WalletContext', () => ({
  useWallet: () => ({ wallet }),
}));

vi.mock('@/contexts/SafeAreaContext', () => ({
  useInsets: () => ({ top: 0, bottom: 0, left: 0, right: 0 }),
}));

vi.mock('@tauri-apps/plugin-os', () => ({
  platform: () => 'macos',
}));

vi.mock('@/components/Layout', () => ({
  default: ({ children }: { children: React.ReactNode }) => (
    <div>{children}</div>
  ),
}));

// The header's own back logic isn't under test, so it's replaced with a button
// that calls the `back` it's given.
vi.mock('@/components/Header', () => ({
  default: ({ title, back }: { title: string; back?: () => void }) => (
    <header>
      <h1>{title}</h1>
      {back && (
        <button type='button' onClick={back}>
          Header Back
        </button>
      )}
    </header>
  ),
}));

// Each page is a marker, so these tests check which page renders, not its
// settings, and nothing loads the Tauri bindings.
function stub(name: string) {
  return () => <div data-testid='page'>{name}</div>;
}

vi.mock('./settings/GeneralSettings', () => ({
  GeneralSettings: stub('general'),
}));
vi.mock('./settings/AppearanceSettings', () => ({
  AppearanceSettings: stub('appearance'),
}));
vi.mock('./settings/PermissionsSettings', () => ({
  PermissionsSettings: stub('permissions'),
}));
vi.mock('./settings/SecuritySettings', () => ({
  SecuritySettings: stub('security'),
}));
vi.mock('./settings/TransactionSettings', () => ({
  TransactionSettings: stub('transactions'),
}));
vi.mock('./settings/NetworkSettings', () => ({
  NetworkSettings: stub('network'),
}));
vi.mock('./settings/ConnectionSettings', () => ({
  ConnectionSettings: stub('connections'),
}));
vi.mock('./settings/AboutSettings', () => ({
  AboutSettings: stub('about'),
}));
vi.mock('./settings/WalletSettings', () => ({
  WalletSettings: stub('wallet'),
  WalletSyncSettings: stub('storage'),
}));

const APP_PAGES = [
  'general',
  'appearance',
  'security',
  'permissions',
  'transactions',
  'network',
  'connections',
  'about',
];
const WALLET_PAGES = ['wallet', 'storage'];

function CurrentPath() {
  const location = useLocation();
  const navigate = useNavigate();

  return (
    <>
      <div data-testid='path'>{location.pathname}</div>
      <button type='button' onClick={() => navigate(-1)}>
        Browser Back
      </button>
    </>
  );
}

function renderSettings(initialEntries: string[]) {
  return render(
    <I18nProvider i18n={i18n}>
      <MemoryRouter initialEntries={initialEntries}>
        <Routes>
          <Route path='/' element={<div>Home</div>} />
          <Route path='/settings' element={<Settings />} />
          <Route path='/settings/:section' element={<Settings />} />
        </Routes>
        <CurrentPath />
      </MemoryRouter>
    </I18nProvider>,
  );
}

function sectionLinks() {
  return screen
    .getAllByRole('link')
    .map((link) => link.getAttribute('href')?.replace('/settings/', ''));
}

function linkTo(href: string) {
  const link = screen
    .getAllByRole('link')
    .find((link) => link.getAttribute('href') === href);
  if (!link) throw new Error(`No link to ${href}`);
  return link;
}

function path() {
  return screen.getByTestId('path').textContent;
}

beforeAll(() => {
  i18n.loadAndActivate({ locale: 'en', messages: {} });
});

beforeEach(() => {
  wallet = null;
});

afterEach(cleanup);

describe('Settings', () => {
  it('lists only the app pages while logged out', () => {
    renderSettings(['/settings']);

    expect(sectionLinks()).toEqual(APP_PAGES);
  });

  it('lists the wallet pages under the wallet name while logged in', () => {
    wallet = { name: 'Savings', fingerprint: 1 };
    renderSettings(['/settings']);

    expect(sectionLinks()).toEqual([
      ...APP_PAGES.slice(0, -1),
      ...WALLET_PAGES,
      'about',
    ]);
    expect(screen.getByRole('heading', { name: 'Savings' })).toBeTruthy();
  });

  it.each([...APP_PAGES, ...WALLET_PAGES])(
    'opens the %s page from its link',
    (id) => {
      wallet = { name: 'Savings', fingerprint: 1 };
      renderSettings(['/settings']);

      fireEvent.click(linkTo(`/settings/${id}`));

      expect(path()).toBe(`/settings/${id}`);
      expect(screen.getByTestId('page').textContent).toBe(id);
    },
  );

  it.each(WALLET_PAGES)(
    'redirects /settings/%s to the list while logged out',
    (id) => {
      renderSettings([`/settings/${id}`]);

      expect(path()).toBe('/settings');
      expect(screen.queryByTestId('page')).toBeNull();
    },
  );

  it('redirects an unknown page to the list', () => {
    renderSettings(['/settings/bogus']);

    expect(path()).toBe('/settings');
    expect(sectionLinks()).toEqual(APP_PAGES);
  });

  it('only offers a way back from the list while logged out', () => {
    renderSettings(['/settings']);
    fireEvent.click(screen.getByText('Header Back'));
    expect(path()).toBe('/');

    cleanup();
    wallet = { name: 'Savings', fingerprint: 1 };
    renderSettings(['/settings']);
    expect(screen.queryByText('Header Back')).toBeNull();
  });

  it('goes back to the list entry rather than adding one', () => {
    renderSettings(['/', '/settings']);

    fireEvent.click(linkTo('/settings/general'));
    fireEvent.click(screen.getByText('Header Back'));
    expect(path()).toBe('/settings');

    // The browser's back button then leaves settings, instead of reopening the page.
    fireEvent.click(screen.getByText('Browser Back'));
    expect(path()).toBe('/');
  });

  it('replaces a page opened directly with the list', () => {
    renderSettings(['/', '/settings/general']);

    fireEvent.click(screen.getByRole('button', { name: 'Settings' }));
    expect(path()).toBe('/settings');

    fireEvent.click(screen.getByText('Browser Back'));
    expect(path()).toBe('/');
  });
});
