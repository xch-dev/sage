import { create } from 'zustand';
import {
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
}

export const useWalletState = create<WalletState>()(() => ({
  sync: {
    receive_address: 'Unknown',
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
}));

export function clearState() {
  useWalletState.setState({
    sync: {
      receive_address: 'Unknown',
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
  });
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
