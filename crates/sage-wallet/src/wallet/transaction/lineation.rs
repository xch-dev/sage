use std::collections::HashMap;

use chia::protocol::Bytes32;
use chia_wallet_sdk::driver::{Did, HashedPtr, Nft, OptionContract, SpendContext, StandardLayer};
use indexmap::IndexMap;

use crate::{Wallet, WalletError};

use super::{Action, Id, NewAssets, Selection, TransactionConfig};

#[derive(Debug)]
pub struct Lineation<'a> {
    pub ctx: &'a mut SpendContext,
    pub nfts: IndexMap<Id, Nft<HashedPtr>>,
    pub dids: IndexMap<Id, Did<HashedPtr>>,
    pub options: IndexMap<Id, OptionContract>,
    pub p2: HashMap<Bytes32, StandardLayer>,
}

impl Wallet {
    pub async fn lineate(
        &self,
        ctx: &mut SpendContext,
        selection: &Selection,
        new_assets: &NewAssets,
        tx: &TransactionConfig,
    ) -> Result<(), WalletError> {
        let mut lineation = Lineation {
            ctx,
            nfts: selection.nfts.clone(),
            dids: selection.dids.clone(),
            options: selection.options.clone(),
            p2: HashMap::new(),
        };

        lineation.nfts.extend(new_assets.nfts.clone());
        lineation.dids.extend(new_assets.dids.clone());
        lineation.options.extend(new_assets.options.clone());

        for p2_puzzle_hash in lineation
            .nfts
            .values()
            .map(|nft| nft.info.p2_puzzle_hash)
            .chain(lineation.dids.values().map(|did| did.info.p2_puzzle_hash))
            .chain(
                lineation
                    .options
                    .values()
                    .map(|option| option.info.p2_puzzle_hash),
            )
        {
            let synthetic_key = self.db.synthetic_key(p2_puzzle_hash).await?;

            lineation
                .p2
                .insert(p2_puzzle_hash, StandardLayer::new(synthetic_key));
        }

        for (index, action) in tx.actions.iter().enumerate() {
            action.lineate(&mut lineation, index)?;

            for p2_puzzle_hash in lineation
                .nfts
                .values()
                .map(|nft| nft.info.p2_puzzle_hash)
                .chain(lineation.dids.values().map(|did| did.info.p2_puzzle_hash))
                .chain(
                    lineation
                        .options
                        .values()
                        .map(|option| option.info.p2_puzzle_hash),
                )
            {
                if lineation.p2.contains_key(&p2_puzzle_hash) {
                    continue;
                }

                if !self.db.is_p2_puzzle_hash(p2_puzzle_hash).await? {
                    continue;
                }

                let synthetic_key = self.db.synthetic_key(p2_puzzle_hash).await?;

                lineation
                    .p2
                    .insert(p2_puzzle_hash, StandardLayer::new(synthetic_key));
            }
        }

        Ok(())
    }
}
