//! How each endpoint behaves for each kind of wallet: BLS, vault and watch. The notes are added
//! to the endpoints' `OpenAPI` descriptions, and `docs/wallet-kinds.md` explains them in more
//! detail. Every endpoint must be listed, so that a new one has its behavior decided.

/// Endpoints that don't use the open wallet, or that manage wallets by fingerprint.
const GLOBAL: &str =
    "Doesn't depend on the open wallet, so it's the same for BLS, vault and watch wallets.";

/// Endpoints that read the open wallet's synced data.
const READ: &str =
    "Works the same for BLS, vault and watch wallets, using what the open wallet has synced.";

/// Endpoints that build a transaction from the open wallet's coins.
const SPEND: &str = "BLS wallets sign with their derived keys. Vaults spend through the vault singleton, \
     signed by their attached custody signers after clear signing checks the spend, and change goes \
     back to the vault. A vault can only have one unconfirmed spend, so another is refused until it \
     confirms. Watch wallets can't sign, and return a wallet error.";

/// Endpoints that spend NFTs, DIDs, options, CAT issuance or clawbacks.
const ASSET_SPEND: &str = "BLS wallets sign with their derived keys. Vaults spend the asset through the vault \
     singleton with their custody signers, as for XCH and CAT sends (vault support for these assets \
     is added by #953). Watch wallets can't sign, and return a wallet error.";

/// Endpoints that make, take or cancel offers.
const OFFER_SPEND: &str = "BLS wallets sign with their derived keys. Vaults make offers from pre-split coins \
     that the offer spends without the vault's signature, and take and cancel them through the vault \
     (added by #955). Watch wallets can't sign, and return a wallet error.";

/// Endpoints for vault recovery and rekeying.
const VAULT_RECOVERY: &str = "Only for vault wallets, added by #954. BLS and watch wallets return an error \
     that the wallet isn't a vault.";

/// Endpoints that only work for vault wallets.
const VAULT_ONLY: &str = "Only for vault wallets. BLS and watch wallets return an error that the wallet \
     isn't a vault.";

/// Endpoints that only work for watch wallets.
const WATCH_ONLY: &str = "Only for watch wallets. BLS and vault wallets return an error that the wallet \
     isn't a watch wallet.";

/// The behavior of an endpoint for each wallet kind, or `None` if it hasn't been decided.
pub(crate) fn wallet_kind_notes(endpoint: &str) -> Option<&'static str> {
    Some(match endpoint {
        "login"
        | "logout"
        | "resync"
        | "generate_mnemonic"
        | "import_wallet"
        | "import_watch_wallet"
        | "import_vault"
        | "find_vault"
        | "delete_wallet"
        | "delete_database"
        | "rename_wallet"
        | "set_wallet_emoji"
        | "get_wallet"
        | "get_wallets"
        | "get_key"
        | "get_keys"
        | "add_key"
        | "delete_key"
        | "get_version"
        | "get_peers"
        | "remove_peer"
        | "add_peer"
        | "set_discover_peers"
        | "set_mempool_discovery"
        | "set_target_peers"
        | "set_network"
        | "set_network_override"
        | "get_networks"
        | "get_network"
        | "set_delta_sync"
        | "set_delta_sync_override"
        | "set_transaction_expiration"
        | "get_user_themes"
        | "get_user_theme"
        | "save_user_theme"
        | "delete_user_theme"
        | "get_xch_usd_price"
        | "combine_offers"
        | "resolve_handle"
        | "get_contacts"
        | "get_contact"
        | "add_contact"
        | "update_contact"
        | "delete_contact"
        | "export_contacts"
        | "import_contacts" => GLOBAL,

        "get_secret_key" => {
            "Returns the secrets of a keychain key by its fingerprint. A vault or watch wallet's own \
             fingerprint isn't a keychain key, so it has no secrets. A vault's signers are keychain \
             keys, and their secrets are returned by their own fingerprints."
        }

        "get_wallet_address" => {
            "BLS wallets return their current receive address, or the change address if one is set. \
             Vaults return their first address, which never changes. Watch wallets return their \
             first watched address."
        }

        "get_sync_status" => {
            "The receive address is a BLS wallet's current address, a vault's first address, or a \
             watch wallet's first watched address. For a vault, the unhardened derivation index is \
             the number of its addresses, by P2 singleton nonce, and the hardened index is 0. Both \
             are 0 for a watch wallet."
        }

        "get_derivations" => {
            "BLS wallets return their derived addresses and synthetic public keys. Vaults return \
             their P2 singleton addresses, whose index is their nonce, with no public key, and no \
             hardened addresses. Watch wallets have no derivations, so the list is empty."
        }

        "increase_derivation_index" => {
            "BLS wallets derive addresses up to the index. Vaults add P2 singleton addresses up to \
             the nonce, and return an error if hardened addresses are asked for explicitly. Watch \
             wallets don't derive addresses, and return a wallet error."
        }

        "check_address" => {
            "Whether the address belongs to the open wallet: a BLS wallet's derived address, one of \
             a vault's addresses, or a watched address."
        }

        "set_change_address" => {
            "Only BLS wallets can set a change address. A vault's change always goes back to the \
             vault, and watch wallets never make change, so setting one returns an error, although \
             clearing one is allowed."
        }

        "get_database_stats"
        | "perform_database_maintenance"
        | "get_handle"
        | "get_handles"
        | "lookup_address_handles"
        | "lookup_address_contacts"
        | "get_are_coins_spendable"
        | "get_spendable_coin_count"
        | "get_coins_by_ids"
        | "get_coins"
        | "get_cats"
        | "get_all_cats"
        | "get_token"
        | "get_dids"
        | "get_minter_did_ids"
        | "get_options"
        | "get_option"
        | "get_pending_transactions"
        | "get_transaction"
        | "get_transactions"
        | "get_nft_collections"
        | "get_nft_collection"
        | "get_nfts"
        | "get_nft"
        | "get_nft_icon"
        | "get_nft_thumbnail"
        | "get_nft_metadata_updater"
        | "get_nft_data"
        | "is_asset_owned"
        | "resync_cat"
        | "update_cat"
        | "update_did"
        | "update_option"
        | "update_nft"
        | "update_nft_collection"
        | "redownload_nft"
        | "view_offer"
        | "import_offer"
        | "get_offers"
        | "get_offers_for_asset"
        | "get_offer"
        | "delete_offer"
        | "view_coin_spends" => READ,

        "submit_transaction" => {
            "Submits an already signed transaction, so it works the same for BLS, vault and watch \
             wallets."
        }

        "send_xch" | "bulk_send_xch" | "combine" | "split" | "auto_combine_xch"
        | "auto_combine_cat" | "send_cat" | "bulk_send_cat" | "multi_send"
        | "create_transaction" => SPEND,

        "sign_coin_spends" => {
            "BLS wallets sign with their derived keys. Vaults only sign with their custody signers' \
             keys for a spend of the vault coin through its custody, which clear signing \
             checks first: spends that change the vault's custody or melt it, ask a vault key to sign \
             for any other coin, or include the vault coin more than once are refused. Watch wallets \
             can't sign, and return a wallet error."
        }

        "issue_cat" | "create_did" | "bulk_mint_nfts" | "transfer_nfts" | "add_nft_uri"
        | "assign_nfts_to_did" | "transfer_dids" | "normalize_dids" | "mint_option"
        | "transfer_options" | "exercise_options" | "cancel_options" | "finalize_clawback"
        | "claim_clawback" => ASSET_SPEND,

        "make_offer" | "take_offer" | "cancel_offer" | "cancel_offers" => OFFER_SPEND,

        "get_vault"
        | "attach_vault_signer"
        | "export_vault_config"
        | "import_vault_config"
        | "rebuild_vault_config" => VAULT_ONLY,

        "start_vault_recovery"
        | "cancel_vault_recovery"
        | "finish_vault_recovery"
        | "rekey_vault" => VAULT_RECOVERY,

        "mint_vault" => {
            "Only BLS wallets can mint a vault, since the mint is paid for from the open wallet's \
             XCH and the custody key is derived from its key. Vault and watch wallets return a \
             wallet error."
        }

        "add_watch_addresses" | "remove_watch_addresses" => WATCH_ONLY,

        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use super::*;

    #[test]
    fn test_every_endpoint_has_wallet_kind_notes() {
        let endpoints: IndexMap<String, bool> =
            serde_json::from_str(include_str!("../../sage-api/endpoints.json"))
                .expect("endpoints.json is valid");

        let missing: Vec<&String> = endpoints
            .keys()
            .filter(|endpoint| wallet_kind_notes(endpoint).is_none())
            .collect();

        assert!(
            missing.is_empty(),
            "decide how these endpoints behave for each wallet kind: {missing:?}"
        );
    }
}
