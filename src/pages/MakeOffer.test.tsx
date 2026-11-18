// @vitest-environment jsdom

import { i18n } from '@lingui/core';
import { I18nProvider } from '@lingui/react';
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
} from '@testing-library/react';
import { ReactNode, useEffect, useRef } from 'react';
import { MemoryRouter } from 'react-router-dom';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';

interface DialogProps {
  fee: string;
  splitNftOffers: boolean;
  preSplitFee: string | null;
}

const dialog = vi.hoisted(() => ({ props: null as DialogProps | null }));
// Whether each fee input, by its label, allows an automatic fee.
const feeInputs = vi.hoisted(() => new Map<string, boolean>());
const progress = vi.hoisted(() => ({
  clearOfferState: null as ((offers: string[]) => void) | null,
}));

vi.mock('@/bindings', () => ({
  commands: {},
  events: { syncEvent: { listen: vi.fn(() => Promise.resolve(vi.fn())) } },
}));
vi.mock('@/components/selectors/AssetSelector', () => ({
  AssetSelector: () => null,
}));
vi.mock('@/components/Container', () => ({
  default: ({ children }: { children: ReactNode }) => <div>{children}</div>,
}));
vi.mock('@/components/ExpirationInput', () => ({
  ExpirationInput: () => null,
}));
vi.mock('@/components/Header', () => ({ default: () => null }));
vi.mock('@/components/ReadOnlyButton', () => ({ ReadOnlyButton: () => null }));
vi.mock('@/components/ui/masked-input', () => ({
  // Named like the real input, which defaults to "Network fee amount".
  FeeAmountInput: (props: {
    'aria-label'?: string;
    allowAuto?: boolean;
    value?: string;
    onValueChange?: (
      values: { value: string },
      sourceInfo: { source: 'prop' | 'event' },
    ) => void;
  }) => <MockFeeInput {...props} />,
}));
vi.mock('@/hooks/useErrors', () => ({
  useErrors: () => ({ addError: vi.fn() }),
}));
vi.mock('@/hooks/useDefaultOfferExpiry', () => ({
  useDefaultOfferExpiry: () => ({ expiry: { enabled: false } }),
}));
vi.mock('@/components/dialogs/MakeOfferConfirmationDialog', () => ({
  MakeOfferConfirmationDialog: (props: DialogProps) => {
    dialog.props = props;
    return null;
  },
}));
vi.mock('@/components/dialogs/OfferCreationProgressDialog', () => ({
  OfferCreationProgressDialog: (props: {
    clearOfferState: (offers: string[]) => void;
  }) => {
    progress.clearOfferState = props.clearOfferState;
    return null;
  },
}));

import { useOfferState } from '@/state';
import { MakeOffer } from './MakeOffer';

beforeAll(() => {
  i18n.loadAndActivate({ locale: 'en', messages: {} });
});

// Like react-number-format, reports a change when the value prop changes.
function MockFeeInput(props: {
  'aria-label'?: string;
  allowAuto?: boolean;
  value?: string;
  onValueChange?: (
    values: { value: string },
    sourceInfo: { source: 'prop' | 'event' },
  ) => void;
}) {
  const { value = '', onValueChange } = props;
  const previous = useRef(value);
  const label = props['aria-label'] ?? 'Network fee amount';
  feeInputs.set(label, props.allowAuto ?? true);

  useEffect(() => {
    if (previous.current === value) return;
    previous.current = value;
    onValueChange?.({ value }, { source: 'prop' });
  }, [value, onValueChange]);

  return (
    <input
      aria-label={label}
      value={value}
      onChange={(e) =>
        onValueChange?.({ value: e.target.value }, { source: 'event' })
      }
    />
  );
}

afterEach(() => {
  cleanup();
  useOfferState.setState(null);
  dialog.props = null;
  feeInputs.clear();
  localStorage.clear();
});

function renderWithOfferedNfts(count: number | string[]) {
  const nfts =
    typeof count === 'number'
      ? Array.from({ length: count }, (_, i) => `nft${i}`)
      : count;

  useOfferState.setState({
    fee: '',
    expiration: null,
    offered: {
      tokens: [],
      nfts,
      options: [],
    },
    requested: { tokens: [], nfts: [], options: [] },
    splitNftOffers: false,
    preSplit: false,
    preSplitFee: '',
  });

  renderPage();
}

function renderPage() {
  render(
    <I18nProvider i18n={i18n}>
      <MemoryRouter>
        <MakeOffer />
      </MemoryRouter>
    </I18nProvider>,
  );
}

function radio(name: string) {
  return screen.getByRole('radio', { name }) as HTMLInputElement;
}

function toggle(name: string) {
  return screen.getByRole('switch', { name }) as HTMLButtonElement;
}

describe('MakeOffer settings', () => {
  it('only asks about separate NFT offers when offering several NFTs', () => {
    renderWithOfferedNfts(1);
    expect(screen.queryByRole('group', { name: 'NFT offers' })).toBeNull();

    cleanup();
    renderWithOfferedNfts(2);
    expect(screen.getByRole('group', { name: 'NFT offers' })).toBeTruthy();
    expect(radio('One offer').checked).toBe(true);
  });

  it('passes the coin splitting and NFT offer choices to the dialog', () => {
    renderWithOfferedNfts(2);
    expect(dialog.props?.preSplitFee).toBeNull();
    expect(dialog.props?.splitNftOffers).toBe(false);

    expect(screen.queryByLabelText('Split fee')).toBeNull();
    fireEvent.click(toggle('Split coins first'));
    expect(dialog.props?.preSplitFee).toBe('');
    expect(screen.getByLabelText('Split fee')).toBeTruthy();

    fireEvent.click(radio('One offer per NFT'));
    expect(dialog.props?.splitNftOffers).toBe(true);
    expect(dialog.props?.preSplitFee).toBeNull();
    expect(toggle('Split coins first').disabled).toBe(true);
    expect(toggle('Split coins first').getAttribute('aria-checked')).toBe(
      'false',
    );

    fireEvent.click(radio('One offer'));
    expect(dialog.props?.splitNftOffers).toBe(false);
    expect(dialog.props?.preSplitFee).toBe('');
  });

  it("only lets the split fee be automatic, since an offer's cost isn't known", () => {
    renderWithOfferedNfts(1);
    fireEvent.click(toggle('Split coins first'));

    expect(feeInputs.get('Network fee amount')).toBe(false);
    expect(feeInputs.get('Split fee')).toBe(true);
  });

  it("doesn't count an NFT row that hasn't been filled in", () => {
    renderWithOfferedNfts(['nft0', '']);

    expect(screen.queryByRole('group', { name: 'NFT offers' })).toBeNull();
    expect(toggle('Split coins first').disabled).toBe(false);
  });

  it('passes each typed fee to the dialog', () => {
    renderWithOfferedNfts(2);
    fireEvent.change(screen.getByLabelText('Network fee amount'), {
      target: { value: '0.25' },
    });
    fireEvent.click(toggle('Split coins first'));
    fireEvent.change(screen.getByLabelText('Split fee'), {
      target: { value: '0.0001' },
    });

    expect(dialog.props?.fee).toBe('0.25');
    expect(dialog.props?.preSplitFee).toBe('0.0001');

    // The split fee comes back after splitting the NFT offers and undoing it.
    fireEvent.click(radio('One offer per NFT'));
    expect(dialog.props?.preSplitFee).toBeNull();
    fireEvent.click(radio('One offer'));
    expect(dialog.props?.preSplitFee).toBe('0.0001');
    expect(dialog.props?.fee).toBe('0.25');
  });

  it('keeps the split choices after leaving the page and coming back', () => {
    renderWithOfferedNfts(2);
    fireEvent.click(radio('One offer per NFT'));
    cleanup();

    renderPage();
    expect(radio('One offer per NFT').checked).toBe(true);
    expect(dialog.props?.splitNftOffers).toBe(true);

    fireEvent.click(radio('One offer'));
    fireEvent.click(toggle('Split coins first'));
    fireEvent.change(screen.getByLabelText('Split fee'), {
      target: { value: '0.0002' },
    });
    cleanup();

    renderPage();
    expect(toggle('Split coins first').getAttribute('aria-checked')).toBe(
      'true',
    );
    expect((screen.getByLabelText('Split fee') as HTMLInputElement).value).toBe(
      '0.0002',
    );
    expect(dialog.props?.preSplitFee).toBe('0.0002');
  });

  it('splits coins by default, paying the default fee', () => {
    localStorage.setItem('defaultFee', JSON.stringify({ fee: '0.001' }));
    renderPage();

    expect(toggle('Split coins first').getAttribute('aria-checked')).toBe(
      'true',
    );
    expect((screen.getByLabelText('Split fee') as HTMLInputElement).value).toBe(
      '0.001',
    );
    expect(dialog.props?.preSplitFee).toBe('0.001');
  });

  it('splits coins by default with an automatic fee', () => {
    renderPage();

    expect(toggle('Split coins first').getAttribute('aria-checked')).toBe(
      'true',
    );
    // The split is a complete transaction, so its fee can be calculated from its cost.
    expect(dialog.props?.preSplitFee).toBe('auto:standard');
  });

  it("doesn't split coins when the setting is off", () => {
    localStorage.setItem('default-offer-split', 'false');
    renderPage();

    expect(toggle('Split coins first').getAttribute('aria-checked')).toBe(
      'false',
    );
    expect(dialog.props?.preSplitFee).toBeNull();
  });

  it('clears the offer on Cancel', () => {
    renderWithOfferedNfts(1);
    act(() => useOfferState.setState({ fee: '0.5' }));

    fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(useOfferState.getState()).toBeNull();
  });

  it('clears the offer on Cancel with a split fee set', () => {
    renderWithOfferedNfts(1);
    act(() =>
      useOfferState.setState({ preSplit: true, preSplitFee: '0.0002' }),
    );

    fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(useOfferState.getState()).toBeNull();
  });

  it('clears the offer on Done after creating it', () => {
    renderWithOfferedNfts(1);
    act(() =>
      useOfferState.setState({
        fee: '0.5',
        preSplit: true,
        preSplitFee: '0.0002',
      }),
    );

    act(() => progress.clearOfferState?.(['offer1']));
    expect(useOfferState.getState()).toBeNull();
  });

  it('shows the stored network fee when coming back', () => {
    renderWithOfferedNfts(1);
    fireEvent.change(screen.getByLabelText('Network fee amount'), {
      target: { value: '0.5' },
    });
    cleanup();

    renderPage();
    expect(
      (screen.getByLabelText('Network fee amount') as HTMLInputElement).value,
    ).toBe('0.5');
  });

  it('sets and clears the expiration', () => {
    renderWithOfferedNfts(0);
    expect(toggle('Expiration').getAttribute('aria-checked')).toBe('false');

    fireEvent.click(toggle('Expiration'));
    expect(useOfferState.getState()?.expiration).toEqual({
      days: '1',
      hours: '0',
      minutes: '0',
    });

    fireEvent.click(toggle('Expiration'));
    expect(useOfferState.getState()?.expiration).toBeNull();
  });
});
