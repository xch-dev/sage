import { create } from 'zustand';
import {
  Assets,
  CoinRecord,
  commands,
  events,
  GetNftStatusResponse,
  GetSyncStatusResponse,
} from './bindings';

export interface WalletState {
  sync: GetSyncStatusResponse;
  nfts: GetNftStatusResponse;
  coins: CoinRecord[];
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

export const useWalletState = create<WalletState>()(() => defaultState());
export const useOfferState = create<OfferState>()(() => defaultOffer());

export function clearState() {
  useWalletState.setState(defaultState());
  useOfferState.setState(defaultOffer());
}

export function clearOffer() {
  useOfferState.setState(defaultOffer());
}

export async function fetchState() {
  await Promise.all([updateCoins(), updateSyncStatus(), updateNftStatus()]);
}

function updateCoins() {
  commands
    .getXchCoins({})
    .then((data) =>
      useWalletState.setState({
        coins: data.coins,
      }),
    )
    .catch((error) => console.error(error));
}

function updateSyncStatus() {
  commands
    .getSyncStatus({})
    .then((sync) => useWalletState.setState({ sync }))
    .catch((error) => console.error(error));
}

function updateNftStatus() {
  commands
    .getNftStatus({})
    .then((nfts) => useWalletState.setState({ nfts }))
    .catch((error) => console.error(error));
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
  await commands.login({ fingerprint });
  await fetchState();
}

export async function logoutAndUpdateState(): Promise<void> {
  clearState();
  await commands.logout({});
}

export function defaultState(): WalletState {
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
  };
}

export function defaultOffer(): OfferState {
  return {
    offered: {
      xch: '',
      cats: [],
      nfts: [],
    },
    requested: {
      xch: '',
      cats: [],
      nfts: [],
    },
    fee: '',
    expiration: null,
  };
}

export function isDefaultOffer(offer: OfferState): boolean {
  return (
    offer.offered.xch === '' &&
    offer.offered.cats.length === 0 &&
    offer.offered.nfts.length === 0 &&
    offer.requested.xch === '' &&
    offer.requested.cats.length === 0 &&
    offer.requested.nfts.length === 0 &&
    offer.fee === '' &&
    offer.expiration === null
  );
}
