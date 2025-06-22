use std::collections::HashMap;

use chia::{
    bls::{
        master_to_wallet_hardened_intermediate, master_to_wallet_unhardened_intermediate, sign,
        DerivableKey, PublicKey, SecretKey,
    },
    protocol::{Bytes32, SpendBundle},
    puzzles::DeriveSynthetic,
};
use chia_wallet_sdk::signer::{AggSigConstants, RequiredSignature};
use clvmr::Allocator;
use itertools::Itertools;

use crate::WalletError;

use super::Wallet;

impl Wallet {
    pub async fn sign_transaction(
        &self,
        spend_bundle: SpendBundle,
        agg_sig_constants: &AggSigConstants,
        master_sk: SecretKey,
        partial: bool,
    ) -> Result<SpendBundle, WalletError> {
        let input_coin_spends = spend_bundle
            .coin_spends
            .iter()
            .filter(|cs| cs.coin.parent_coin_info != Bytes32::default())
            .cloned()
            .collect_vec();

        let required_signatures = RequiredSignature::from_coin_spends(
            &mut Allocator::new(),
            &input_coin_spends,
            agg_sig_constants,
        )?;

        let mut indices = HashMap::new();

        for required in &required_signatures {
            let RequiredSignature::Bls(required) = required else {
                return Err(WalletError::SecpNotSupported);
            };
            let pk = required.public_key;
            let Some(info) = self.db.synthetic_key_info(pk).await? else {
                if partial {
                    continue;
                }
                return Err(WalletError::UnknownPublicKey);
            };
            indices.insert(pk, info);
        }

        let unhardened_intermediate_sk = master_to_wallet_unhardened_intermediate(&master_sk);
        let hardened_intermediate_sk = master_to_wallet_hardened_intermediate(&master_sk);

        let secret_keys: HashMap<PublicKey, SecretKey> = indices
            .iter()
            .map(|(pk, info)| {
                let sk = if info.hardened {
                    hardened_intermediate_sk.derive_hardened(info.index)
                } else {
                    unhardened_intermediate_sk.derive_unhardened(info.index)
                }
                .derive_synthetic();

                (*pk, sk)
            })
            .collect();

        let mut aggregated_signature = spend_bundle.aggregated_signature;

        for required in required_signatures {
            let RequiredSignature::Bls(required) = required else {
                return Err(WalletError::SecpNotSupported);
            };
            let Some(sk) = secret_keys.get(&required.public_key).cloned() else {
                continue;
            };
            aggregated_signature += &sign(&sk, required.message());
        }

        Ok(SpendBundle::new(
            spend_bundle.coin_spends,
            aggregated_signature,
        ))
    }
}
