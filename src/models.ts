export enum WalletKind {
  Cold = 'cold',
  Hot = 'hot',
}

export enum DerivationMode {
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
}
