use std::ops::Add;

use chia::protocol::{Bytes32, Coin, Program};
use chia_wallet_sdk::driver::{Cat, Nft, Offer};
use indexmap::IndexMap;

use crate::{Wallet, WalletError};

#[derive(Debug, Default, Clone)]
pub struct OfferAmounts {
    pub xch: u64,
    pub cats: IndexMap<Bytes32, u64>,
}

impl Add for OfferAmounts {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut cats = self.cats;

        for (asset_id, amount) in rhs.cats {
            *cats.entry(asset_id).or_insert(0) += amount;
        }

        Self {
            xch: self.xch + rhs.xch,
            cats,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OfferCoins {
    pub xch: Vec<Coin>,
    pub cats: IndexMap<Bytes32, Vec<Cat>>,
    pub nfts: IndexMap<Bytes32, Nft<Program>>,
}

impl OfferCoins {
    pub fn nonce(&self) -> Bytes32 {
        let mut coin_ids = Vec::new();

        for coin in &self.xch {
            coin_ids.push(coin.coin_id());
        }

        for cat_coins in self.cats.values() {
            for cat in cat_coins {
                coin_ids.push(cat.coin.coin_id());
            }
        }

        for nft in self.nfts.values() {
            coin_ids.push(nft.coin.coin_id());
        }

        Offer::nonce(coin_ids)
    }

    pub fn primary_coin_ids(&self) -> Vec<Bytes32> {
        let mut primary_coins = Vec::new();

        if let Some(coin) = self.xch.first() {
            primary_coins.push(coin.coin_id());
        }

        for cat_coins in self.cats.values() {
            if let Some(cat) = cat_coins.first() {
                primary_coins.push(cat.coin.coin_id());
            }
        }

        for nft in self.nfts.values() {
            primary_coins.push(nft.coin.coin_id());
        }

        primary_coins
    }
}

impl Wallet {
    pub async fn fetch_offer_coins(
        &self,
        total_amounts: &OfferAmounts,
        nft_ids: Vec<Bytes32>,
    ) -> Result<OfferCoins, WalletError> {
        // Select XCH coins.
        let xch = if total_amounts.xch > 0 {
            self.select_p2_coins(total_amounts.xch as u128).await?
        } else {
            Vec::new()
        };

        // Select CAT coins.
        let mut cats = IndexMap::new();

        for (&asset_id, &amount) in &total_amounts.cats {
            if amount == 0 {
                continue;
            }

            cats.insert(
                asset_id,
                self.select_cat_coins(asset_id, amount as u128).await?,
            );
        }

        // Fetch NFT coins.
        let mut nfts = IndexMap::new();

        for nft_id in nft_ids {
            let Some(nft) = self.db.spendable_nft(nft_id).await? else {
                return Err(WalletError::MissingNft(nft_id));
            };

            nfts.insert(nft_id, nft);
        }

        Ok(OfferCoins { xch, cats, nfts })
    }
}
