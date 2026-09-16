mod app;
mod bridge;
mod environment;
mod wallet;

pub(crate) use app::*;
pub(crate) use bridge::*;
pub(crate) use environment::*;
pub(crate) use wallet::*;

#[cfg(test)]
mod tests {
    use serde::de::DeserializeOwned;
    use serde_json::{Value, json};

    use super::*;

    fn assert_unknown_field_rejected<T: DeserializeOwned>(value: Value) {
        let error = serde_json::from_value::<T>(value)
            .err()
            .expect("boundary input must reject unknown fields");

        assert!(
            error.to_string().contains("unknown field"),
            "unexpected decode error: {error}"
        );
    }

    #[test]
    fn chip0002_action_inputs_reject_unknown_top_level_fields() {
        assert_unknown_field_rejected::<WalletGetKeyParams>(json!({ "fingerprint": 1 }));
        assert_unknown_field_rejected::<WalletGetSecretKeyParams>(json!({ "fingerprint": 1 }));
        assert_unknown_field_rejected::<WalletGetPublicKeysParams>(json!({ "future": true }));
        assert_unknown_field_rejected::<WalletFilterUnlockedCoinsParams>(json!({
            "coinNames": ["coin-id"],
            "future": true
        }));
        assert_unknown_field_rejected::<WalletGetAssetCoinsParams>(json!({
            "type": null,
            "assetId": null,
            "future": true
        }));
        assert_unknown_field_rejected::<WalletGetAssetBalanceParams>(json!({
            "type": null,
            "assetId": null,
            "future": true
        }));
        assert_unknown_field_rejected::<WalletSignCoinSpendsParams>(json!({
            "coinSpends": [],
            "future": true
        }));
        assert_unknown_field_rejected::<WalletSignMessageParams>(json!({
            "message": "00",
            "publicKey": "public-key",
            "future": true
        }));
        assert_unknown_field_rejected::<WalletSendTransactionParams>(json!({
            "spendBundle": {
                "coin_spends": [],
                "aggregated_signature": "signature"
            },
            "future": true
        }));
    }

    #[test]
    fn chip0002_nested_signing_inputs_reject_unknown_fields() {
        assert_unknown_field_rejected::<WalletSignCoinSpendsParams>(json!({
            "coinSpends": [{
                "coin": {
                    "parent_coin_info": "parent",
                    "puzzle_hash": "puzzle-hash",
                    "amount": 1,
                    "future": true
                },
                "puzzle_reveal": "puzzle",
                "solution": "solution"
            }]
        }));

        assert_unknown_field_rejected::<WalletSendTransactionParams>(json!({
            "spendBundle": {
                "coin_spends": [],
                "aggregated_signature": "signature",
                "future": true
            }
        }));
    }
}
