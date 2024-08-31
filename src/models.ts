export enum WalletKind {
  Cold = 'cold',
  Hot = 'hot',
}

export enum DerivationMode {
  Automatic = 'automatic',
  Manual = 'manual',
}

export enum PeerMode {
  Automatic = 'automatic',
  Manual = 'manual',
}

export interface WalletInfo {
  name: string;
  fingerprint: number;
  kind: WalletKind;
}

export interface WalletConfig {
  name: string;
  derivation_mode: DerivationMode;
  derivation_batch_size: number;
  derivation_index: number;
}

export interface NetworkConfig {
  network_id: string;
  target_peers: number;
  peer_mode: PeerMode;
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
