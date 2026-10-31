use std::collections::HashMap;

use chia_wallet_sdk::{
    chia::{
        bls::{
            DerivableKey, aggregate_verify, master_to_wallet_hardened_intermediate,
            master_to_wallet_unhardened_intermediate, sign,
        },
        puzzle_types::DeriveSynthetic,
    },
    prelude::*,
};
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
        let required_signatures =
            required_signatures(&spend_bundle.coin_spends, agg_sig_constants)?;

        let mut derivations = HashMap::new();

        for required in &required_signatures {
            let RequiredSignature::Bls(required) = required else {
                continue;
            };
            let Some(derivation) = self.db.derivation(required.public_key).await? else {
                continue;
            };
            derivations.insert(required.public_key, derivation);
        }

        let unhardened_intermediate_sk = master_to_wallet_unhardened_intermediate(&master_sk);
        let hardened_intermediate_sk = master_to_wallet_hardened_intermediate(&master_sk);

        let secret_keys: HashMap<PublicKey, SecretKey> = derivations
            .iter()
            .map(|(public_key, derivation)| {
                let secret_key = if derivation.is_hardened {
                    hardened_intermediate_sk.derive_hardened(derivation.derivation_index)
                } else {
                    unhardened_intermediate_sk.derive_unhardened(derivation.derivation_index)
                }
                .derive_synthetic();

                (*public_key, secret_key)
            })
            .collect();

        let mut aggregated_signature = spend_bundle.aggregated_signature;

        for required in required_signatures {
            let RequiredSignature::Bls(required) = required else {
                continue;
            };

            let sk = if required.public_key == master_sk.public_key() {
                master_sk.clone()
            } else if let Some(sk) = secret_keys.get(&required.public_key).cloned() {
                sk
            } else {
                if partial {
                    continue;
                }
                return Err(WalletError::UnknownPublicKey);
            };

            aggregated_signature += &sign(&sk, required.message());
        }

        Ok(SpendBundle::new(
            spend_bundle.coin_spends,
            aggregated_signature,
        ))
    }
}

/// Whether the signature already covers every BLS signature the coin spends require.
pub fn is_fully_signed(
    coin_spends: &[CoinSpend],
    signature: &Signature,
    agg_sig_constants: &AggSigConstants,
) -> Result<bool, WalletError> {
    let pairs = required_signatures(coin_spends, agg_sig_constants)?
        .into_iter()
        .filter_map(|required| match required {
            RequiredSignature::Bls(required) => Some((required.public_key, required.message())),
            RequiredSignature::Secp(_) => None,
        })
        .collect_vec();

    Ok(aggregate_verify(signature, pairs))
}

fn required_signatures(
    coin_spends: &[CoinSpend],
    agg_sig_constants: &AggSigConstants,
) -> Result<Vec<RequiredSignature>, WalletError> {
    let input_coin_spends = coin_spends
        .iter()
        .filter(|cs| cs.coin.parent_coin_info != Bytes32::default())
        .cloned()
        .collect_vec();

    Ok(RequiredSignature::from_coin_spends(
        &mut Allocator::new(),
        &input_coin_spends,
        agg_sig_constants,
    )?)
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use crate::TestWallet;

    use super::*;

    #[test(tokio::test)]
    async fn test_is_fully_signed() -> anyhow::Result<()> {
        let alice = TestWallet::new(1000).await?;

        let coin_spends = alice
            .wallet
            .send_xch(vec![(alice.puzzle_hash, 1000)], 0, vec![], None)
            .await?;

        assert!(!is_fully_signed(
            &coin_spends,
            &Signature::default(),
            &alice.agg_sig
        )?);

        let spend_bundle = alice
            .wallet
            .sign_transaction(
                SpendBundle::new(coin_spends, Signature::default()),
                &alice.agg_sig,
                alice.master_sk.clone(),
                false,
            )
            .await?;

        assert!(is_fully_signed(
            &spend_bundle.coin_spends,
            &spend_bundle.aggregated_signature,
            &alice.agg_sig
        )?);

        // Adding the same signatures again makes the bundle invalid.
        let doubled =
            spend_bundle.aggregated_signature.clone() + &spend_bundle.aggregated_signature;

        assert!(!is_fully_signed(
            &spend_bundle.coin_spends,
            &doubled,
            &alice.agg_sig
        )?);

        Ok(())
    }
}
