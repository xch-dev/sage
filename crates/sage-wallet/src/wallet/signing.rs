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

            if could_be_signed_message(&required) {
                return Err(WalletError::UnsafeSignedMessage);
            }

            aggregated_signature += &sign(&sk, required.message());
        }

        Ok(SpendBundle::new(
            spend_bundle.coin_spends,
            aggregated_signature,
        ))
    }
}

/// Whether signing this would also produce a valid signed message, in the format
/// `sign_message_*` produces: a signature over `tree_hash(("Chia Signed Message" . message))`.
///
/// `AGG_SIG_UNSAFE` signs its message as is, so a coin spend that asks for one over that
/// tree hash gets a signed message without the user ever seeing the sign-message prompt.
/// The hash can't be told apart from any other 32 bytes without its preimage, which the
/// spend doesn't have to include, so every 32-byte `AGG_SIG_UNSAFE` message is refused.
/// Messages of any other length can't be a signed message, and are signed as before.
fn could_be_signed_message(required: &RequiredBlsSignature) -> bool {
    // Only `AGG_SIG_UNSAFE` has no domain string.
    required.domain_string.is_none() && required.raw_message.len() == 32
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
    use chia_wallet_sdk::{chia::bls::master_to_wallet_unhardened, clvm_traits::clvm_quote};
    use test_log::test;

    use crate::TestWallet;

    use super::*;

    /// A spend of a throwaway coin, as a dApp could supply, whose puzzle outputs a single
    /// `AGG_SIG_UNSAFE` condition.
    fn agg_sig_unsafe_spend(public_key: PublicKey, message: Bytes) -> anyhow::Result<CoinSpend> {
        let mut ctx = SpendContext::new();
        let puzzle = ctx.alloc(&clvm_quote!(vec![AggSigUnsafe::new(public_key, message)]))?;
        let coin = Coin::new(Bytes32::new([1; 32]), ctx.tree_hash(puzzle).into(), 1);
        Ok(CoinSpend::new(
            coin,
            ctx.serialize(&puzzle)?,
            ctx.serialize(&NodePtr::NIL)?,
        ))
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
                    matches!(result, Err(WalletError::UnsafeSignedMessage)),
                    "{result:?}"
                );
            }
        }

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_refuse_any_32_byte_unsafe_message() -> anyhow::Result<()> {
        let alice = TestWallet::new(0).await?;
        let coin_spend =
            agg_sig_unsafe_spend(synthetic_key(&alice).public_key(), vec![7; 32].into())?;

        assert!(matches!(
            sign_spend(&alice, coin_spend, false).await,
            Err(WalletError::UnsafeSignedMessage)
        ));

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_sign_ordinary_unsafe_message() -> anyhow::Result<()> {
        let alice = TestWallet::new(0).await?;
        let synthetic_sk = synthetic_key(&alice);

        for message in [Bytes::default(), vec![7; 31].into(), vec![7; 33].into()] {
            let coin_spend = agg_sig_unsafe_spend(synthetic_sk.public_key(), message.clone())?;
            let spend_bundle = sign_spend(&alice, coin_spend, false).await?;

            assert_eq!(
                spend_bundle.aggregated_signature,
                sign(&synthetic_sk, message)
            );
            assert!(is_fully_signed(
                &spend_bundle.coin_spends,
                &spend_bundle.aggregated_signature,
                &alice.agg_sig
            )?);
        }

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_skip_unsafe_message_for_unknown_key() -> anyhow::Result<()> {
        let alice = TestWallet::new(0).await?;
        let other = SecretKey::from_seed(&[42; 32]).public_key();
        let coin_spend = agg_sig_unsafe_spend(other, vec![7; 32].into())?;

        // Only signatures Sage would actually produce are refused.
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
