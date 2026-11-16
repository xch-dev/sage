use serde::{Deserialize, Serialize};
use specta::Type;

macro_rules! define_wallet_capabilities {
    (
        $visibility:vis enum $name:ident {
            $(
                $variant:ident => $key:literal $(| $alias:literal)*
            ),* $(,)?
        }
    ) => {
        #[derive(
            Debug,
            Clone,
            Copy,
            Serialize,
            Deserialize,
            Type,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
        )]
        $visibility enum $name {
            $(
                #[serde(rename = $key $(, alias = $alias)*)]
                #[specta(rename = $key)]
                $variant,
            )*
        }

        impl $name {
            pub const ALL: &'static [Self] = &[
                $(Self::$variant),*
            ];

            pub fn key(self) -> &'static str {
                match self {
                    $(Self::$variant => $key),*
                }
            }

            pub fn from_key(key: &str) -> Option<Self> {
                match key {
                    $($key $(| $alias)* => Some(Self::$variant),)*
                    _ => None,
                }
            }
        }
    };
}

define_wallet_capabilities! {
    pub enum WalletCapability {
        WalletGetWallet => "wallet.get_wallet" | "wallet.get_key",
        WalletGetSecretKey => "wallet.get_secret_key",
        WalletSendXch => "wallet.send_xch",
        WalletSendXchAutoSubmit => "wallet.send_xch_auto_submit",
        WalletGetSyncStatus => "wallet.get_sync_status",
        WalletGetVersion => "wallet.get_version",
        WalletGetXchUsdPrice => "wallet.get_xch_usd_price",
        WalletCheckAddress => "wallet.check_address",
        WalletResolveHandle => "wallet.resolve_handle",
        WalletGetHandles => "wallet.get_handles",
        WalletGetContacts => "wallet.get_contacts",
        WalletFilterUnlockedCoins => "wallet.filter_unlocked_coins",
        WalletGetAssetCoins => "wallet.get_asset_coins",
        WalletGetAssetBalance => "wallet.get_asset_balance",
        WalletSignCoinSpends => "wallet.sign_coin_spends",
        WalletSignMessage => "wallet.sign_message",
        WalletSendTransaction => "wallet.send_transaction",
        WalletGetPublicKeys => "wallet.get_public_keys",
        WalletGetDerivations => "wallet.get_derivations",
        WalletGetSpendableCoinCount => "wallet.get_spendable_coin_count",
        WalletGetCoinsByIds => "wallet.get_coins_by_ids",
        WalletGetCoins => "wallet.get_coins",
        WalletGetPendingTransactions => "wallet.get_pending_transactions",
        WalletGetTransaction => "wallet.get_transaction",
        WalletGetTransactions => "wallet.get_transactions",
        WalletGetWallets => "wallet.get_wallets",
        WalletGetWalletAddress => "wallet.get_wallet_address",
        WalletGetDatabaseStats => "wallet.get_database_stats",
        WalletGetContact => "wallet.get_contact",
        WalletAddContact => "wallet.add_contact",
        WalletUpdateContact => "wallet.update_contact",
        WalletDeleteContact => "wallet.delete_contact",
        WalletExportContacts => "wallet.export_contacts",
        WalletImportContacts => "wallet.import_contacts",
        WalletGetAreCoinsSpendable => "wallet.get_are_coins_spendable",
        WalletGetCats => "wallet.get_cats",
        WalletGetAllCats => "wallet.get_all_cats",
        WalletGetToken => "wallet.get_token",
        WalletGetDids => "wallet.get_dids",
        WalletGetMinterDidIds => "wallet.get_minter_did_ids",
        WalletGetOptions => "wallet.get_options",
        WalletGetOption => "wallet.get_option",
        WalletGetNftCollections => "wallet.get_nft_collections",
        WalletGetNftCollection => "wallet.get_nft_collection",
        WalletGetNfts => "wallet.get_nfts",
        WalletGetNft => "wallet.get_nft",
        WalletGetNftIcon => "wallet.get_nft_icon",
        WalletGetNftThumbnail => "wallet.get_nft_thumbnail",
        WalletGetNftMetadataUpdater => "wallet.get_nft_metadata_updater",
        WalletGetNftData => "wallet.get_nft_data",
        WalletBulkSendXch => "wallet.bulk_send_xch",
        WalletCombine => "wallet.combine",
        WalletSplit => "wallet.split",
        WalletAutoCombineXch => "wallet.auto_combine_xch",
        WalletAutoCombineCat => "wallet.auto_combine_cat",
        WalletIssueCat => "wallet.issue_cat",
        WalletSendCat => "wallet.send_cat",
        WalletBulkSendCat => "wallet.bulk_send_cat",
        WalletMultiSend => "wallet.multi_send",
        WalletCreateDid => "wallet.create_did",
        WalletBulkMintNfts => "wallet.bulk_mint_nfts",
        WalletTransferNfts => "wallet.transfer_nfts",
        WalletAddNftUri => "wallet.add_nft_uri",
        WalletAssignNftsToDid => "wallet.assign_nfts_to_did",
        WalletTransferDids => "wallet.transfer_dids",
        WalletNormalizeDids => "wallet.normalize_dids",
        WalletMintOption => "wallet.mint_option",
        WalletTransferOptions => "wallet.transfer_options",
        WalletExerciseOptions => "wallet.exercise_options",
        WalletCancelOptions => "wallet.cancel_options",
        WalletFinalizeClawback => "wallet.finalize_clawback",
        WalletClaimClawback => "wallet.claim_clawback",
        WalletCreateTransaction => "wallet.create_transaction",
        WalletViewCoinSpends => "wallet.view_coin_spends",
        WalletSubmitTransaction => "wallet.submit_transaction",
        WalletMakeOffer => "wallet.make_offer",
        WalletTakeOffer => "wallet.take_offer",
        WalletCombineOffers => "wallet.combine_offers",
        WalletViewOffer => "wallet.view_offer",
        WalletImportOffer => "wallet.import_offer",
        WalletGetOffers => "wallet.get_offers",
        WalletGetOffersForAsset => "wallet.get_offers_for_asset",
        WalletGetOffer => "wallet.get_offer",
        WalletDeleteOffer => "wallet.delete_offer",
        WalletCancelOffer => "wallet.cancel_offer",
        WalletCancelOffers => "wallet.cancel_offers",
        WalletGetPeers => "wallet.get_peers",
        WalletGetUserThemes => "wallet.get_user_themes",
        WalletGetUserTheme => "wallet.get_user_theme",
        WalletGetNetworks => "wallet.get_networks",
        WalletGetNetwork => "wallet.get_network",
        WalletResyncCat => "wallet.resync_cat",
        WalletUpdateCat => "wallet.update_cat",
        WalletUpdateDid => "wallet.update_did",
        WalletUpdateOption => "wallet.update_option",
        WalletUpdateNft => "wallet.update_nft",
        WalletUpdateNftCollection => "wallet.update_nft_collection",
        WalletRedownloadNft => "wallet.redownload_nft",
        WalletIsAssetOwned => "wallet.is_asset_owned",
    }
}
