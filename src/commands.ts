import { invoke } from '@tauri-apps/api/core';
import {
  DerivationMode,
  Network,
  NetworkConfig,
  PeerInfo,
  PeerMode,
  SyncInfo,
  WalletConfig,
  WalletInfo,
} from './models';

export async function initialize(): Promise<void> {
  await invoke('initialize');
}

export async function activeWallet(): Promise<WalletInfo | null> {
  return await invoke('active_wallet');
}

export async function networkConfig(): Promise<NetworkConfig> {
  return await invoke('network_config');
}

export async function setTargetPeers(targetPeers: number): Promise<void> {
  await invoke('set_target_peers', { targetPeers });
}

export async function setPeerMode(peerMode: PeerMode): Promise<void> {
  await invoke('set_peer_mode', { peerMode });
}

export async function setNetworkId(networkId: string): Promise<void> {
  await invoke('set_network_id', { networkId });
}

export async function walletConfig(fingerprint: number): Promise<WalletConfig> {
  return await invoke('wallet_config', { fingerprint });
}

export async function networkList(): Promise<Record<string, Network>> {
  return await invoke('network_list');
}

export async function loginWallet(fingerprint: number): Promise<void> {
  await invoke('login_wallet', { fingerprint });
}

export async function logoutWallet(): Promise<void> {
  await invoke('logout_wallet');
}

export async function walletList(): Promise<WalletInfo[]> {
  return await invoke('wallet_list');
}

export async function generateMnemonic(use24Words: boolean): Promise<string> {
  return await invoke('generate_mnemonic', { use24Words });
}

export async function createWallet(
  name: string,
  mnemonic: string,
  saveMnemonic: boolean,
): Promise<void> {
  await invoke('create_wallet', { name, mnemonic, saveMnemonic });
}

export async function importWallet(name: string, key: string): Promise<void> {
  await invoke('import_wallet', { name, key });
}

export async function deleteWallet(fingerprint: number): Promise<void> {
  await invoke('delete_wallet', { fingerprint });
}

export async function renameWallet(
  fingerprint: number,
  name: string,
): Promise<void> {
  await invoke('rename_wallet', { fingerprint, name });
}

export async function setDerivationMode(
  fingerprint: number,
  derivationMode: DerivationMode,
): Promise<void> {
  await invoke('set_derivation_mode', { fingerprint, derivationMode });
}

export async function setDerivationBatchSize(
  fingerprint: number,
  derivationBatchSize: number,
): Promise<void> {
  await invoke('set_derivation_batch_size', {
    fingerprint,
    derivationBatchSize,
  });
}

export async function peerList(): Promise<PeerInfo[]> {
  return await invoke('peer_list');
}

export async function syncInfo(): Promise<SyncInfo> {
  return await invoke('sync_info');
}
