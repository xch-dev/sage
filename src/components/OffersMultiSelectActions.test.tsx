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
import {
  afterEach,
  beforeAll,
  beforeEach,
  describe,
  expect,
  it,
  vi,
} from 'vitest';

const fetchOfferRecords = vi.fn();
const deleteOffers = vi.fn();
const addError = vi.fn();
const toastInfo = vi.fn();
let isTransactionDisabled = false;

vi.mock('@/lib/offers', () => ({
  fetchOfferRecords: (...args: unknown[]) => fetchOfferRecords(...args),
  deleteOffers: (...args: unknown[]) => deleteOffers(...args),
}));

vi.mock('@/contexts/WalletContext', () => ({
  useWallet: () => ({ isTransactionDisabled }),
}));

vi.mock('@/hooks/useErrors', () => ({
  useErrors: () => ({ addError }),
}));

vi.mock('react-toastify', () => ({
  toast: {
    info: (...args: unknown[]) => toastInfo(...args),
    error: vi.fn(),
    success: vi.fn(),
  },
}));

// The real MultiSelectActionBar and dropdown-menu primitives are Radix
// components that require pointer-capture APIs jsdom doesn't implement.
// This component's own logic (what gets fetched/filtered/passed down) is
// what these tests exercise, so the surrounding menu chrome is replaced
// with plain, always-open markup.
vi.mock('./MultiSelectActionBar', () => ({
  MultiSelectActionBar: ({ children }: { children: React.ReactNode }) => (
    <div>{children}</div>
  ),
}));

vi.mock('./ui/dropdown-menu', () => ({
  DropdownMenuItem: ({
    children,
    onClick,
    disabled,
    'aria-label': ariaLabel,
  }: {
    children: React.ReactNode;
    onClick?: (e: React.MouseEvent) => void;
    disabled?: boolean;
    'aria-label'?: string;
  }) => (
    <button
      type='button'
      role='menuitem'
      aria-label={ariaLabel}
      disabled={disabled}
      onClick={onClick as never}
    >
      {children}
    </button>
  ),
  DropdownMenuSeparator: () => <hr />,
}));

let lastCancelFlowProps: {
  open: boolean;
  offers: { offer_id: string }[];
} | null = null;

vi.mock('./dialogs/CancelOffersFlow', () => ({
  CancelOffersFlow: (props: {
    open: boolean;
    offers: { offer_id: string }[];
  }) => {
    lastCancelFlowProps = props;
    return null;
  },
}));

// DeleteOfferDialog pulls in the themed Dialog primitive, which isn't
// relevant to these bulk-cancel tests and drags in a JSON import vitest
// can't transform outside the app's own build pipeline.
vi.mock('./dialogs/DeleteOfferDialog', () => ({
  DeleteOfferDialog: () => null,
}));

const { OffersMultiSelectActions } = await import('./OffersMultiSelectActions');

function renderComponent(selected: string[]) {
  return render(
    <I18nProvider i18n={i18n}>
      <OffersMultiSelectActions
        selected={selected}
        onConfirm={() => undefined}
      />
    </I18nProvider>,
  );
}

beforeAll(() => {
  i18n.loadAndActivate({ locale: 'en', messages: {} });
});

beforeEach(() => {
  fetchOfferRecords.mockReset();
  deleteOffers.mockReset();
  addError.mockReset();
  toastInfo.mockReset();
  isTransactionDisabled = false;
  lastCancelFlowProps = null;
});

afterEach(() => {
  cleanup();
});

function cancelMenuItem() {
  return screen.getByRole('menuitem', { name: /Cancel \d+ selected offers/ });
}

describe('OffersMultiSelectActions bulk cancel', () => {
  it('opens the cancel flow with only the fresh, still-active offers', async () => {
    // Selection includes an offer ('gone') that was deleted from its own
    // row menu after being selected, and one ('done') that completed via a
    // coin_state refresh since selection.
    fetchOfferRecords.mockImplementation(async (ids: string[]) => {
      expect(ids).toEqual(['a', 'gone', 'done']);
      return [
        { offer_id: 'a', status: 'active' },
        { offer_id: 'done', status: 'completed' },
      ];
    });

    renderComponent(['a', 'gone', 'done']);

    await act(async () => {
      fireEvent.click(cancelMenuItem());
    });

    expect(fetchOfferRecords).toHaveBeenCalledTimes(1);
    expect(lastCancelFlowProps?.open).toBe(true);
    expect(lastCancelFlowProps?.offers.map((o) => o.offer_id)).toEqual(['a']);
  });

  it('shows a non-blocking notice and does not open the flow when nothing is active', async () => {
    fetchOfferRecords.mockResolvedValue([
      { offer_id: 'done', status: 'completed' },
    ]);

    renderComponent(['done']);

    await act(async () => {
      fireEvent.click(cancelMenuItem());
    });

    expect(toastInfo).toHaveBeenCalledTimes(1);
    expect(lastCancelFlowProps?.open).toBe(false);
  });

  it('is enabled for any non-empty selection, not just ones known to be active', () => {
    renderComponent(['a', 'b']);

    expect(cancelMenuItem()).not.toHaveProperty('disabled', true);
    // No stale-fetch-on-mount: nothing is fetched until Cancel is clicked.
    expect(fetchOfferRecords).not.toHaveBeenCalled();
  });
});
