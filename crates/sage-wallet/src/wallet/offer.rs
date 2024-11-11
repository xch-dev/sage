use chia::protocol::{Bytes32, Coin, CoinSpend, Program};

use chia_wallet_sdk::{
    AssertPuzzleAnnouncement, Cat, Nft, OfferBuilder, Partial, SpendContext, TradePrice,
};
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

#[derive(Debug, Clone)]
pub struct OfferSpend {
    pub p2_coins: Vec<Coin>,
    pub p2_amount: u64,
    pub cats: Vec<CatOfferSpend>,
    pub nfts: Vec<NftOfferSpend>,
    pub assertions: Vec<AssertPuzzleAnnouncement>,
    pub change_puzzle_hash: Bytes32,
}

#[derive(Debug, Clone)]
pub struct CatOfferSpend {
    pub coins: Vec<Cat>,
    pub amount: u64,
}

#[derive(Debug, Clone)]
pub struct NftOfferSpend {
    pub nft: Nft<Program>,
    pub trade_prices: Vec<TradePrice>,
}
