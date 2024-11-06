use chia::protocol::{Bytes32, CoinSpend, Program};

use chia_wallet_sdk::{OfferBuilder, Partial, SpendContext};
use indexmap::{IndexMap, IndexSet};

#[derive(Debug, Clone)]
pub struct OfferedCoins {
    pub xch: u64,
    pub cats: IndexMap<Bytes32, u64>,
    pub nfts: IndexSet<Bytes32>,
}

#[derive(Debug, Clone)]
pub struct OfferRequest {
    pub xch: u64,
    pub cats: IndexMap<Bytes32, u64>,
    pub nfts: IndexMap<Bytes32, NftOfferDetails>,
}

#[derive(Debug, Clone)]
pub struct NftOfferDetails {
    pub metadata: Program,
    pub metadata_updater_puzzle_hash: Bytes32,
    pub royalty_puzzle_hash: Bytes32,
    pub royalty_ten_thousandths: u16,
}

#[derive(Debug)]
pub struct UnsignedOffer {
    pub ctx: SpendContext,
    pub coin_spends: Vec<CoinSpend>,
    pub builder: OfferBuilder<Partial>,
}
