import { listen } from '@tauri-apps/api/event';
import { create } from 'zustand';
import * as commands from './commands';
import { CoinData, SyncEventData, SyncInfo } from './models';

export interface WalletState {
  syncInfo: SyncInfo;
  coins: CoinData[];
}

export const useWalletState = create<WalletState>()(() => ({
  syncInfo: {
    address: 'Unknown',
    balance: 'Syncing',
    ticker: 'XCH',
    total_coins: 0,
    synced_coins: 0,
  },
  coins: [],
}));

export function clearState() {
  useWalletState.setState({
    syncInfo: {
      address: 'Unknown',
      balance: 'Syncing',
      ticker: 'XCH',
      total_coins: 0,
      synced_coins: 0,
    },
    coins: [],
  });
}

export async function fetchState() {
  await Promise.all([updateCoins(), updateSyncInfo()]);
}

function updateCoins() {
  commands.coinList().then((coins) => {
    useWalletState.setState({
      coins,
    });
  });
}

function updateSyncInfo() {
  commands.syncInfo().then((syncInfo) => {
    useWalletState.setState({
      syncInfo,
    });
  });
}

listen<SyncEventData>('sync', (event) => {
  switch (event.payload.type) {
    case 'coin_update':
      updateCoins();
      updateSyncInfo();
      break;
    case 'puzzle_update':
      updateSyncInfo();
      break;
  }
});

commands.initialize().then(() => {
  commands.activeWallet().then((wallet) => {
    if (wallet) {
      updateCoins();
      updateSyncInfo();
    }
  });
});
