export enum WalletKind {
  Cold = 'cold',
  Hot = 'hot',
}

export interface WalletInfo {
  name: string;
  fingerprint: number;
  kind: WalletKind;
}

export interface WalletConfig {
  name: string;
  derive_automatically: boolean;
  derivation_batch_size: number;
  derivation_index: number;
}

export interface NetworkConfig {
  network_id: string;
  target_peers: number;
  discover_peers: boolean;
}

export interface Network {
  default_port: number;
  genesis_challenge: string;
  agg_sig_me: string | null;
  dns_introducers: string[];
}

export interface PeerInfo {
  ip_addr: string;
  port: number;
  trusted: boolean;
}

export interface SyncInfo {
  xch_balance: string;
  total_coins: number;
  synced_coins: number;
}

export interface CoinData {
  coin_id: string;
  address: string;
  created_height: number | null;
  spent_height: number | null;
  amount: string;
}

export interface DidData {
  encoded_id: string;
  launcher_id: string;
  address: string;
}

export interface NftData {
  encoded_id: string;
  launcher_id: string;
  address: string;
}

export type SyncEventData =
  | { type: 'start'; ip: string }
  | { type: 'stop' | 'subscribed' | 'coin_update' | 'puzzle_update' };
