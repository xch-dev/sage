// @vitest-environment jsdom

import { i18n } from '@lingui/core';
import { I18nProvider } from '@lingui/react';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { ReactNode } from 'react';
import { MemoryRouter } from 'react-router-dom';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';

interface DialogProps {
  splitNftOffers: boolean;
  preSplitFee: string | null;
}

const dialog = vi.hoisted(() => ({ props: null as DialogProps | null }));

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
  FeeAmountInput: (props: { 'aria-label'?: string }) => (
    <input aria-label={props['aria-label']} />
  ),
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
  OfferCreationProgressDialog: () => null,
}));

import { useOfferState } from '@/state';
import { MakeOffer } from './MakeOffer';

beforeAll(() => {
  i18n.loadAndActivate({ locale: 'en', messages: {} });
});

afterEach(() => {
  cleanup();
  useOfferState.setState(null);
  dialog.props = null;
});

function renderWithOfferedNfts(count: number) {
  useOfferState.setState({
    fee: '',
    expiration: null,
    offered: {
      tokens: [],
      nfts: Array.from({ length: count }, (_, i) => `nft${i}`),
      options: [],
    },
    requested: { tokens: [], nfts: [], options: [] },
  });

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
