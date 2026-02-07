use chia_wallet_sdk::{
    driver::{
        FeeAction, MetadataUpdate, MintNftAction, SendAction, TransferNftById, UpdateNftAction,
        UriKind,
    },
    prelude::*,
    puzzles::NFT_METADATA_UPDATER_DEFAULT_HASH,
};
use sage_api::{CreateTransaction, NftUriKind, TransactionResponse};
use sage_wallet::{calculate_memos, Hint};

use crate::{
    parse_amount, parse_any_asset_id, parse_coin_ids, parse_memos, ConfirmationInfo, Result, Sage,
};

impl Sage {
    pub async fn create_transaction(&self, req: CreateTransaction) -> Result<TransactionResponse> {
        let wallet = self.wallet()?;

        let sender_puzzle_hash = wallet.change_p2_puzzle_hash().await?;
        let selected_coin_ids = parse_coin_ids(req.selected_coin_ids)?;

        let mut ctx = SpendContext::new();
        let mut actions = vec![];
        let mut info = ConfirmationInfo::default();

        for action in req.actions {
            match action {
                sage_api::Action::Send(action) => {
                    let receiver_puzzle_hash = self.parse_address(action.address)?;
                    let amount = parse_amount(action.amount)?;
                    let id = parse_id(action.id)?;
                    let hinted = id != Id::Xch;
                    let memos = parse_memos(action.memos)?;

                    let clawback = action.clawback.map(|seconds| {
                        ClawbackV2::new(
                            sender_puzzle_hash,
                            receiver_puzzle_hash,
                            seconds,
                            amount,
                            hinted,
                        )
                    });

                    let memos = calculate_memos(
                        &mut ctx,
                        if let Some(clawback) = clawback {
                            Hint::Clawback(clawback)
                        } else {
                            Hint::None
                        },
                        memos,
                    )?;

                    let puzzle_hash = if let Some(clawback) = clawback {
                        clawback.tree_hash().into()
                    } else {
                        receiver_puzzle_hash
                    };

                    actions.push(Action::Send(SendAction {
                        id,
                        puzzle_hash,
                        amount,
                        memos,
                    }));
                }
                sage_api::Action::MintNft(action) => {
                    let parent_id = parse_id(action.parent_id)?;

                    let mint = self
                        .convert_nft_mint(
                            sage_api::NftMint {
                                address: None,
                                edition_number: action.edition_number,
                                edition_total: action.edition_total,
                                data_hash: action.data_hash,
                                data_uris: action.data_uris,
                                metadata_hash: action.metadata_hash,
                                metadata_uris: action.metadata_uris,
                                license_hash: action.license_hash,
                                license_uris: action.license_uris,
                                royalty_address: action.royalty_address,
                                royalty_ten_thousandths: action.royalty_ten_thousandths,
                            },
                            &mut info,
                        )
                        .await?;

                    let metadata = ctx.alloc_hashed(&mint.metadata)?;

                    actions.push(Action::MintNft(MintNftAction {
                        parent_id,
                        metadata,
                        metadata_updater_puzzle_hash: NFT_METADATA_UPDATER_DEFAULT_HASH.into(),
                        royalty_puzzle_hash: mint.royalty_puzzle_hash.unwrap_or(sender_puzzle_hash),
                        royalty_basis_points: mint.royalty_basis_points,
                        amount: 1,
                    }));
                }
                sage_api::Action::UpdateNft(action) => {
                    let id = parse_id(action.id)?;

                    let mut metadata_update_spends = vec![];

                    for uri in action.new_uris {
                        let spend = MetadataUpdate {
                            kind: match uri.kind {
                                NftUriKind::Data => UriKind::Data,
                                NftUriKind::Metadata => UriKind::Metadata,
                                NftUriKind::License => UriKind::License,
                            },
                            uri: uri.uri,
                        }
                        .spend(&mut ctx)?;

                        metadata_update_spends.push(spend);
                    }

                    let transfer = if let Some(transfer) = action.transfer {
                        let did_id = transfer.did_id.map(parse_id).transpose()?;

                        Some(TransferNftById::new(did_id, vec![]))
                    } else {
                        None
                    };

                    actions.push(Action::UpdateNft(UpdateNftAction {
                        id,
                        metadata_update_spends,
                        transfer,
                    }));
                }
                sage_api::Action::Fee(action) => {
                    actions.push(Action::Fee(FeeAction {
                        amount: parse_amount(action.amount)?,
                    }));
                }
            }
        }

        wallet.spend(&mut ctx, selected_coin_ids, &actions).await?;

        let coin_spends = ctx.take();

        self.transact_with(coin_spends, req.auto_submit, info).await
    }
}

fn parse_id(id: sage_api::Id) -> Result<Id> {
    Ok(match id {
        sage_api::Id::Xch => Id::Xch,
        sage_api::Id::Existing { asset_id } => Id::Existing(parse_any_asset_id(asset_id)?),
        sage_api::Id::New { index } => Id::New(index),
    })
}
