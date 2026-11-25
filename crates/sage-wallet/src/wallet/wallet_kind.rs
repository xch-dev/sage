use chia_wallet_sdk::prelude::*;

/// What a wallet is made of, which determines how its addresses are found and how it signs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalletKind {
    /// A BLS key, whose standard addresses are derived from the unhardened intermediate key.
    Bls { intermediate_pk: PublicKey },
    /// A list of watched p2 puzzle hashes, stored in the database as external p2 puzzles. Their
    /// puzzle reveals are unknown, so the wallet can't derive addresses or spend.
    Watch,
}
