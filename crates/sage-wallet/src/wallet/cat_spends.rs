use chia_wallet_sdk::{
    driver::{Cat, CatSpend, SpendContext, SpendWithConditions, StandardLayer},
    types::Conditions,
};

use crate::WalletError;

use super::Wallet;

impl Wallet {
    /// Spends the CATs with the given conditions. No outputs are created automatically.
    pub(crate) async fn spend_cat_coins(
        &self,
        ctx: &mut SpendContext,
        cats: impl Iterator<Item = (Cat, Conditions)>,
    ) -> Result<(), WalletError> {
        let mut cat_spends = Vec::new();

        for (cat, conditions) in cats {
            // We need to figure out what the synthetic public key is for this CAT coin.
            let synthetic_key = self.db.synthetic_key(cat.p2_puzzle_hash).await?;

            // Create the standard p2 layer for the key.
            let p2 = StandardLayer::new(synthetic_key);

            // Spend the CAT with the given conditions.
            cat_spends.push(CatSpend::new(
                cat,
                p2.spend_with_conditions(ctx, conditions)?,
            ));
        }

        Cat::spend_all(ctx, &cat_spends)?;

        Ok(())
    }
}
