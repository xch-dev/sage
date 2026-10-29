use chia_wallet_sdk::prelude::*;

/// What a wallet is made of, which determines how its addresses are found and how it signs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalletKind {
    /// A BLS key, whose standard addresses are derived from the unhardened intermediate key.
    Bls { intermediate_pk: PublicKey },
}
