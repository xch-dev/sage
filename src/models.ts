export enum WalletKind {
  Cold = 'Cold',
  Hot = 'Hot',
}

export enum DerivationMode {
  Automatic = 'Automatic',
  Manual = 'Manual',
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
}
