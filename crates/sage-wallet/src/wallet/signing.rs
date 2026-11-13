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

use crate::{WalletError, is_placeholder_spend};

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

            if is_agg_sig_unsafe(&required) {
                return Err(WalletError::AggSigUnsafe);
            }

            aggregated_signature += &sign(&sk, required.message());
        }

        Ok(SpendBundle::new(
            spend_bundle.coin_spends,
            aggregated_signature,
        ))
    }
}

/// Whether this is an `AGG_SIG_UNSAFE` signature, which Sage refuses to produce with its keys.
///
/// `AGG_SIG_UNSAFE` signs its message as is, without the coin or network data the other
/// `AGG_SIG_*` kinds append, so the signature is valid for anything else over the same bytes.
/// A dApp-supplied spend could use it to get a signed message (a signature over
/// `tree_hash(("Chia Signed Message" . message))`) without the sign-message prompt, or an
/// `AGG_SIG_ME` signature for one of the user's coins on another network. Sage's own spends
/// never use it, so it's refused outright rather than by matching message formats.
fn is_agg_sig_unsafe(required: &RequiredBlsSignature) -> bool {
    // Only `AGG_SIG_UNSAFE` has no domain string.
    required.domain_string.is_none()
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
        .filter(|cs| !is_placeholder_spend(cs))
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
    use chia_wallet_sdk::chia::bls::master_to_wallet_unhardened;
    use test_log::test;

    use crate::{TestWallet, quoted_spend};

    use super::*;

    /// A spend of a throwaway coin, as a dApp could supply, whose puzzle outputs a single
    /// `AGG_SIG_UNSAFE` condition.
    fn agg_sig_unsafe_spend(public_key: PublicKey, message: Bytes) -> anyhow::Result<CoinSpend> {
        quoted_spend(
            Bytes32::new([1; 32]),
            1,
            Conditions::new().with(AggSigUnsafe::new(public_key, message)),
        )
    }

    fn synthetic_key(alice: &TestWallet) -> SecretKey {
        master_to_wallet_unhardened(&alice.master_sk, 0).derive_synthetic()
    }

    async fn sign_spend(
        alice: &TestWallet,
        coin_spend: CoinSpend,
        partial: bool,
    ) -> Result<SpendBundle, WalletError> {
        alice
            .wallet
            .sign_transaction(
                SpendBundle::new(vec![coin_spend], Signature::default()),
                &alice.agg_sig,
                alice.master_sk.clone(),
                partial,
            )
            .await
    }

    #[test(tokio::test)]
    async fn test_refuse_unsafe_signed_message() -> anyhow::Result<()> {
        let alice = TestWallet::new(0).await?;
        let synthetic_sk = synthetic_key(&alice);

        let signed_message: Bytes32 = (
            "Chia Signed Message",
            Bytes::from(b"Send me all your XCH".to_vec()),
        )
            .tree_hash()
            .into();

        for public_key in [synthetic_sk.public_key(), alice.master_sk.public_key()] {
            let coin_spend = agg_sig_unsafe_spend(public_key, signed_message.to_vec().into())?;

            for partial in [false, true] {
                let result = sign_spend(&alice, coin_spend.clone(), partial).await;
                assert!(
                    matches!(result, Err(WalletError::AggSigUnsafe)),
                    "{result:?}"
                );
            }
        }

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_refuse_any_unsafe_message_for_owned_key() -> anyhow::Result<()> {
        let alice = TestWallet::new(0).await?;
        let synthetic_sk = synthetic_key(&alice);

        // An `AGG_SIG_ME` message for one of the user's coins on another network.
        let mut agg_sig_me = vec![7; 64];
        agg_sig_me.extend(MAINNET_CONSTANTS.agg_sig_me_additional_data.to_vec());

        for message in [Bytes::default(), vec![7; 32].into(), agg_sig_me.into()] {
            let coin_spend = agg_sig_unsafe_spend(synthetic_sk.public_key(), message)?;
            let result = sign_spend(&alice, coin_spend, false).await;
            assert!(
                matches!(result, Err(WalletError::AggSigUnsafe)),
                "{result:?}"
            );
        }

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_skip_unsafe_message_for_unknown_key() -> anyhow::Result<()> {
        let alice = TestWallet::new(0).await?;
        let other = SecretKey::from_seed(&[42; 32]).public_key();
        let coin_spend = agg_sig_unsafe_spend(other, vec![7; 32].into())?;

        // Only keys Sage holds are refused; others are left for whoever holds them.
        let spend_bundle = sign_spend(&alice, coin_spend.clone(), true).await?;
        assert_eq!(spend_bundle.aggregated_signature, Signature::default());

        assert!(matches!(
            sign_spend(&alice, coin_spend, false).await,
            Err(WalletError::UnknownPublicKey)
        ));

        Ok(())
    }

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
