import { create } from 'zustand';
import {
  Amount,
  commands,
  events,
  GetSyncStatusResponse,
  KeyInfo,
} from './bindings';
import { CustomError } from './contexts/ErrorContext';

export interface WalletState {
  sync: GetSyncStatusResponse;
}

export interface AssetInput {
  xch: string;
  cats: CatInput[];
  nfts: string[];
}

export interface CatInput {
  assetId: string;
  amount: string;
}

export interface OfferState {
  offered: Assets;
  requested: Assets;
  fee: string;
  expiration: OfferExpiration | null;
}

export interface Assets {
  tokens: TokenAmount[];
  nfts: string[];
  options: string[];
}

export interface FeePolicyInput {
  recipient: string;
  fee_basis_points: string;
  min_fee: string;
  allow_zero_price: boolean;
  allow_revoke_fee_bypass: boolean;
}

export interface TokenAmount {
  asset_id: string | null;
  amount: Amount;
  fee_policy?: FeePolicyInput | null;
}

export interface OfferExpiration {
  days: string;
  hours: string;
  minutes: string;
}

export interface ReturnValue {
  status: 'success' | 'completed' | 'cancelled';
  data?: string;
}

export interface NavigationStore {
  returnValues: Record<string, ReturnValue>;
  setReturnValue: (pageId: string, value: ReturnValue) => void;
}

export const useWalletState = create<WalletState>(() => defaultState());
export const useOfferState = create<OfferState | null>(() => null);
export const useNavigationStore = create<NavigationStore>((set) => ({
  returnValues: {},
  setReturnValue: (pageId, value) =>
    set((state) => ({
      returnValues: { ...state.returnValues, [pageId]: value },
    })),
}));

export function clearState() {
  useWalletState.setState(defaultState());
  useOfferState.setState(null);
}

export async function fetchState() {
  await Promise.all([updateSyncStatus()]);
}

let updateSyncStatusPromise: Promise<void> | null = null;

export function updateSyncStatus() {
  // Prevent multiple concurrent calls
  if (updateSyncStatusPromise) {
    return;
  }

  updateSyncStatusPromise = commands
    .getKey({})
    .then((key) => {
      // Only call getSyncStatus if key and key.key are not null
      if (key && key.key) {
        return commands.getSyncStatus({});
      }
      return null;
    })
    .then((sync) => {
      if (sync) {
        useWalletState.setState({ sync });
      }
    })
    .catch((error) => console.error(error))
    .finally(() => {
      updateSyncStatusPromise = null;
    });
}

events.syncEvent.listen((event) => {
  switch (event.payload.type) {
    case 'coin_state':
    case 'derivation':
    case 'puzzle_batch_synced':
    case 'nft_data':
      updateSyncStatus();
      break;
  }
});

export async function loginAndUpdateState(
  fingerprint: number,
  onError?: (error: CustomError) => void,
): Promise<void> {
  try {
    await commands.login({ fingerprint });
    await fetchState();
  } catch (error) {
    if (onError) {
      onError(error as CustomError);
    } else {
      console.error(error);
    }
    throw error;
  }
}

// Create a separate function to handle wallet state updates
let setWalletState: ((wallet: KeyInfo | null) => void) | null = null;

export function initializeWalletState(
  setter: (wallet: KeyInfo | null) => void,
) {
  setWalletState = setter;
}

export async function logoutAndUpdateState(): Promise<void> {
  clearState();
  if (setWalletState) {
    setWalletState(null);
  }
  await commands.logout({});
}

export function defaultState(): WalletState {
  return {
    sync: {
      receive_address: 'Unknown',
      burn_address: 'Unknown',
      selectable_balance: '0',
      unit: {
        ticker: 'XCH',
        precision: 12,
      },
      total_coins: 0,
      synced_coins: 0,
      unhardened_derivation_index: 0,
      hardened_derivation_index: 0,
      checked_files: 0,
      total_files: 0,
      database_size: 0,
    },
  };
}
