import { create } from 'zustand';
import { CoinData, commands, events, SyncInfo } from './bindings';

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
    if (coins.status === 'error') {
      console.error(coins.error);
      return;
    }
    useWalletState.setState({
      coins: coins.data,
    });
  });
}

function updateSyncInfo() {
  commands.syncInfo().then((syncInfo) => {
    if (syncInfo.status === 'error') {
      console.error(syncInfo.error);
      return;
    }
    useWalletState.setState({
      syncInfo: syncInfo.data,
    });
  });
}

events.syncEvent.listen((event) => {
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
      fetchState();
    }
  });
});

export async function loginAndUpdateState(fingerprint: number): Promise<void> {
  await commands.loginWallet(fingerprint);
  await fetchState();
}

export async function logoutAndUpdateState(): Promise<void> {
  clearState();
  await commands.logoutWallet();
}
