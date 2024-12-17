use std::collections::HashMap;

use chia::{
    bls::{
        master_to_wallet_unhardened_intermediate, sign, DerivableKey, PublicKey, SecretKey,
        Signature,
    },
    protocol::{CoinSpend, SpendBundle},
    puzzles::DeriveSynthetic,
};
use chia_wallet_sdk::{AggSigConstants, Offer, RequiredSignature};
use clvmr::Allocator;

use crate::WalletError;

use super::{UnsignedMakeOffer, UnsignedTakeOffer, Wallet};

impl Wallet {
    pub async fn sign_make_offer(
        &self,
        info: UnsignedMakeOffer,
        agg_sig_constants: &AggSigConstants,
        master_sk: SecretKey,
    ) -> Result<Offer, WalletError> {
        let UnsignedMakeOffer {
            mut ctx,
            coin_spends,
            builder,
        } = info;

        let spend_bundle = self
            .sign_transaction(coin_spends, agg_sig_constants, master_sk, false)
            .await?;

        Ok(builder.bundle(&mut ctx, spend_bundle)?)
    }

    pub async fn sign_take_offer(
        &self,
        info: UnsignedTakeOffer,
        agg_sig_constants: &AggSigConstants,
        master_sk: SecretKey,
    ) -> Result<SpendBundle, WalletError> {
        let UnsignedTakeOffer {
            coin_spends,
            builder,
        } = info;

        let spend_bundle = self
            .sign_transaction(coin_spends, agg_sig_constants, master_sk, false)
            .await?;

        Ok(builder.bundle(spend_bundle))
    }

    pub async fn sign_transaction(
        &self,
        coin_spends: Vec<CoinSpend>,
        agg_sig_constants: &AggSigConstants,
        master_sk: SecretKey,
        partial: bool,
    ) -> Result<SpendBundle, WalletError> {
        let required_signatures = RequiredSignature::from_coin_spends(
            &mut Allocator::new(),
            &coin_spends,
            agg_sig_constants,
        )?;

        let mut indices = HashMap::new();

        for required in &required_signatures {
            let pk = required.public_key();
            let Some(index) = self.db.synthetic_key_index(pk).await? else {
                if partial {
                    continue;
                }
                return Err(WalletError::UnknownPublicKey);
            };
            indices.insert(pk, index);
        }

        let intermediate_sk = master_to_wallet_unhardened_intermediate(&master_sk);

        let secret_keys: HashMap<PublicKey, SecretKey> = indices
            .iter()
            .map(|(pk, index)| {
                let sk = intermediate_sk.derive_unhardened(*index).derive_synthetic();
                (*pk, sk)
            })
            .collect();

        let mut aggregated_signature = Signature::default();

        for required in required_signatures {
            let sk = secret_keys[&required.public_key()].clone();
            aggregated_signature += &sign(&sk, required.final_message());
        }

        Ok(SpendBundle::new(coin_spends, aggregated_signature))
    }
}
