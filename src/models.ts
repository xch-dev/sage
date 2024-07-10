export enum WalletKind {
  Cold = 'Cold',
  Hot = 'Hot',
}

export enum DerivationMode {
  Reuse = 'Reuse',
  Cycle = 'Cycle',
  Generate = 'Generate',
}

export interface WalletInfo {
  name: string;
  fingerprint: number;
  kind: WalletKind;
}

export interface WalletConfig {
  name: string;
  derivation_mode: DerivationMode;
}
