import { create } from 'zustand';
import {
  Assets,
  commands,
  events,
  GetSyncStatusResponse,
  KeyInfo,
} from './bindings';

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

function updateSyncStatus() {
  commands
    .getSyncStatus({})
    .then((sync) => useWalletState.setState({ sync }))
    .catch((error) => console.error(error));
}

events.syncEvent.listen((event) => {
  switch (event.payload.type) {
    case 'coin_state':
      updateSyncStatus();
      break;
    case 'derivation':
      updateSyncStatus();
      break;
    case 'puzzle_batch_synced':
      updateSyncStatus();
      break;
  }
});

export async function loginAndUpdateState(fingerprint: number): Promise<void> {
  await commands.login({ fingerprint });
  await fetchState();
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
      balance: '0',
      unit: {
        ticker: 'XCH',
        decimals: 12,
      },
      total_coins: 0,
      synced_coins: 0,
      unhardened_derivation_index: 0,
      hardened_derivation_index: 0,
    },
  };
}
