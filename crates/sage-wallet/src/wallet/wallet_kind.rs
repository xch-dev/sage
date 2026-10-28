use chia_wallet_sdk::{driver::P2Singleton, prelude::*};

/// What a wallet is made of, which determines how its addresses are found and whether it can
/// derive new ones.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WalletKind {
    /// A BLS key, whose standard addresses are derived from the unhardened intermediate key.
    Bls { intermediate_pk: PublicKey },
    /// A vault singleton, which receives at `P2Singleton` addresses.
    Vault { launcher_id: Bytes32 },
    /// A fixed list of p2 puzzle hashes, whose puzzle reveals are unknown.
    Watch { p2_puzzle_hashes: Vec<Bytes32> },
}

impl WalletKind {
    pub fn is_bls(&self) -> bool {
        matches!(self, Self::Bls { .. })
    }

    /// The unhardened intermediate key, if this wallet derives its addresses.
    pub fn intermediate_pk(&self) -> Option<PublicKey> {
        match self {
            Self::Bls { intermediate_pk } => Some(*intermediate_pk),
            Self::Vault { .. } | Self::Watch { .. } => None,
        }
    }

    /// The fixed address that receives payments and change for wallets without derivations.
    /// Vaults use their `P2Singleton` address with nonce 0, and watch wallets their first
    /// address.
    pub fn fixed_p2_puzzle_hash(&self) -> Option<Bytes32> {
        match self {
            Self::Bls { .. } => None,
            Self::Vault { launcher_id } => {
                Some(P2Singleton::new(*launcher_id, 0).tree_hash().into())
            }
            Self::Watch { p2_puzzle_hashes } => p2_puzzle_hashes.first().copied(),
        }
    }
}
