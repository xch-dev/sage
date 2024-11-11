import { create } from 'zustand';
import {
  Assets,
  CoinRecord,
  commands,
  events,
  NftStatus,
  SyncStatus,
} from './bindings';

export interface WalletState {
  sync: SyncStatus;
  nfts: NftStatus;
  coins: CoinRecord[];
  offerAssets: Assets;
  requestedAssets: Assets;
  offerFee: string;
}

export const useWalletState = create<WalletState>()(() => defaultState());

export function clearState() {
  useWalletState.setState(defaultState());
}

export async function fetchState() {
  await Promise.all([updateCoins(), updateSyncStatus(), updateNftStatus()]);
}

function updateCoins() {
  commands.getCoins().then((coins) => {
    if (coins.status === 'error') {
      console.error(coins.error);
      return;
    }
    useWalletState.setState({
      coins: coins.data,
    });
  });
}

function updateSyncStatus() {
  commands.getSyncStatus().then((sync) => {
    if (sync.status === 'error') {
      console.error(sync.error);
      return;
    }
    useWalletState.setState({
      sync: sync.data,
    });
  });
}

function updateNftStatus() {
  commands.getNftStatus().then((nfts) => {
    if (nfts.status === 'error') {
      console.error(nfts.error);
      return;
    }
    useWalletState.setState({
      nfts: nfts.data,
    });
  });
}

events.syncEvent.listen((event) => {
  switch (event.payload.type) {
    case 'coin_state':
      updateCoins();
      updateSyncStatus();
      updateNftStatus();
      break;
    case 'derivation':
      updateSyncStatus();
      break;
    case 'puzzle_batch_synced':
      updateSyncStatus();
      updateNftStatus();
      break;
    case 'nft_data':
      updateNftStatus();
      break;
  }
});

export async function loginAndUpdateState(fingerprint: number): Promise<void> {
  await commands.loginWallet(fingerprint);
  await fetchState();
}

export async function logoutAndUpdateState(): Promise<void> {
  clearState();
  await commands.logoutWallet();
}

function defaultState(): WalletState {
  return {
    sync: {
      receive_address: 'Unknown',
      burn_address: 'Unknown',
      balance: 'Syncing',
      unit: {
        ticker: 'XCH',
        decimals: 12,
      },
      total_coins: 0,
      synced_coins: 0,
    },
    nfts: {
      nfts: 0,
      visible_nfts: 0,
      collections: 0,
      visible_collections: 0,
    },
    coins: [],
    offerAssets: {
      xch: '',
      nfts: [],
      cats: [],
    },
    requestedAssets: {
      xch: '',
      nfts: [],
      cats: [],
    },
    offerFee: '',
  };
}

export function clearOffer() {
  useWalletState.setState({
    offerAssets: {
      xch: '',
      cats: [],
      nfts: [],
    },
    requestedAssets: {
      xch: '',
      cats: [],
      nfts: [],
    },
    offerFee: '',
  });
}
