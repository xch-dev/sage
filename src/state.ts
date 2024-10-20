import { create } from 'zustand';
import { CoinRecord, commands, events, SyncStatus } from './bindings';

export interface WalletState {
  sync: SyncStatus;
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
    coins: [],
  });
}

export async function fetchState() {
  await Promise.all([updateCoins(), updateSyncStatus()]);
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

events.syncEvent.listen((event) => {
  switch (event.payload.type) {
    case 'coin_update':
      updateCoins();
      updateSyncStatus();
      break;
    case 'puzzle_update':
      updateSyncStatus();
      break;
    case 'transaction_update':
      updateCoins();
      updateSyncStatus();
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
