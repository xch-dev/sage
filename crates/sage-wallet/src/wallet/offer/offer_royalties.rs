use chia_wallet_sdk::{
    chia::puzzle_types::offer::{Payment, SettlementPaymentsSolution},
    driver::calculate_nft_royalty,
    prelude::*,
    puzzles::SETTLEMENT_PAYMENT_HASH,
};
use indexmap::IndexMap;

use crate::WalletError;

/// Royalty payments the taker has to make for NFTs offered by makers.
///
/// Each maker's NFT spend into settlement commits to a list of trade prices, and its royalty
/// transfer program asserts one payment of `floor(price * basis_points / 10000)` per trade price.
/// Those trade prices are per maker, so they can't be reconstructed from the aggregate offer's
/// totals. Settlement payments must be positive, so a trade price whose royalty rounds to zero
/// can never be satisfied.
pub(crate) fn offered_nft_royalty_actions(
    ctx: &mut SpendContext,
    offer: &Offer,
) -> Result<Vec<Action>, WalletError> {
    let mut actions = Vec::new();

    for (&launcher_id, nft) in &offer.offered_coins().nfts {
        let royalty = RoyaltyInfo::new(
            launcher_id,
            nft.info.royalty_puzzle_hash,
            nft.info.royalty_basis_points,
        );

        for trade_price in committed_trade_prices(ctx, offer, nft)? {
            let id = settlement_asset_id(offer, trade_price.puzzle_hash)
                .ok_or(WalletError::MissingAsset(trade_price.puzzle_hash))?;
            let amount = calculate_nft_royalty(trade_price.amount, royalty.basis_points);

            if amount == 0 {
                return Err(WalletError::ZeroRoyaltyPayment(launcher_id));
            }

            actions.push(Action::settle(id, royalty.payment(ctx, amount)?));
        }
    }

    Ok(actions)
}

/// Trade prices for an NFT the taker supplies to a buyer.
///
/// Buyers pay the royalty for a requested NFT inside their own offer, so the NFT has to reveal
/// trade prices whose royalties match those payments exactly.
pub(crate) fn requested_nft_trade_prices(
    prepaid_royalties: &IndexMap<Bytes32, Vec<(Bytes32, Payment)>>,
    nft: &NftInfo,
) -> Vec<TradePrice> {
    if nft.royalty_basis_points == 0 {
        return Vec::new();
    }

    let basis_points = u64::from(nft.royalty_basis_points);

    prepaid_royalties
        .get(&nft.launcher_id)
        .into_iter()
        .flatten()
        .filter(|(_, payment)| payment.puzzle_hash == nft.royalty_puzzle_hash)
        .map(|(settlement_puzzle_hash, payment)| {
            TradePrice::new(
                (payment.amount * 10_000).div_ceil(basis_points),
                *settlement_puzzle_hash,
            )
        })
        .collect()
}

/// Royalty payments already made by settlement spends in the offer, keyed by nonce (the NFT's
/// launcher id), along with the settlement puzzle hash of the asset they're paid in.
pub(crate) fn prepaid_royalties(
    allocator: &mut Allocator,
    offer: &Offer,
) -> Result<IndexMap<Bytes32, Vec<(Bytes32, Payment)>>, WalletError> {
    let mut prepaid: IndexMap<Bytes32, Vec<(Bytes32, Payment)>> = IndexMap::new();

    for coin_spend in &offer.spend_bundle().coin_spends {
        let puzzle = coin_spend.puzzle_reveal.to_clvm(allocator)?;
        let puzzle = Puzzle::parse(allocator, puzzle);
        let solution = coin_spend.solution.to_clvm(allocator)?;

        let (settlement_puzzle_hash, settlement_solution) =
            if SettlementLayer::parse_puzzle(allocator, puzzle)?.is_some() {
                (SETTLEMENT_PAYMENT_HASH.into(), solution)
            } else if let Some(parsed) = Cat::parse(allocator, coin_spend.coin, puzzle, solution)?
                && SettlementLayer::parse_puzzle(allocator, parsed.p2_puzzle)?.is_some()
            {
                let info = CatInfo::new(
                    parsed.cat.info.asset_id,
                    parsed.cat.info.hidden_puzzle_hash,
                    SETTLEMENT_PAYMENT_HASH.into(),
                );
                (info.puzzle_hash().into(), parsed.p2_solution)
            } else {
                continue;
            };

        let solution = SettlementPaymentsSolution::from_clvm(allocator, settlement_solution)?;

        for notarized_payment in solution.notarized_payments {
            prepaid.entry(notarized_payment.nonce).or_default().extend(
                notarized_payment
                    .payments
                    .into_iter()
                    .map(|payment| (settlement_puzzle_hash, payment)),
            );
        }
    }

    Ok(prepaid)
}

fn committed_trade_prices(
    allocator: &mut Allocator,
    offer: &Offer,
    nft: &Nft,
) -> Result<Vec<TradePrice>, WalletError> {
    let Some(parent_spend) = offer
        .spend_bundle()
        .coin_spends
        .iter()
        .find(|coin_spend| coin_spend.coin.coin_id() == nft.coin.parent_coin_info)
    else {
        return Err(WalletError::MissingNftCoin(nft.coin.parent_coin_info));
    };

    let puzzle = parent_spend.puzzle_reveal.to_clvm(allocator)?;
    let puzzle = Puzzle::parse(allocator, puzzle);
    let solution = parent_spend.solution.to_clvm(allocator)?;

    let Some((_, p2_puzzle, p2_solution)) =
        Nft::parse(allocator, parent_spend.coin, puzzle, solution)?
    else {
        return Err(WalletError::MissingNftCoin(parent_spend.coin.coin_id()));
    };

    let output = run_puzzle(allocator, p2_puzzle.ptr(), p2_solution)?;
    let conditions = Vec::<Condition>::from_clvm(allocator, output)?;

    Ok(conditions
        .into_iter()
        .find_map(|condition| match condition {
            Condition::TransferNft(transfer) => Some(transfer.trade_prices),
            _ => None,
        })
        .unwrap_or_default())
}

fn settlement_asset_id(offer: &Offer, settlement_puzzle_hash: Bytes32) -> Option<Id> {
    if settlement_puzzle_hash == SETTLEMENT_PAYMENT_HASH.into() {
        return Some(Id::Xch);
    }

    offer
        .requested_payments()
        .cats
        .keys()
        .chain(offer.offered_coins().cats.keys())
        .find(|&&asset_id| {
            let hidden_puzzle_hash = offer
                .asset_info()
                .cat(asset_id)
                .and_then(|info| info.hidden_puzzle_hash);
            CatInfo::new(asset_id, hidden_puzzle_hash, SETTLEMENT_PAYMENT_HASH.into()).puzzle_hash()
                == settlement_puzzle_hash.into()
        })
        .map(|&asset_id| Id::Existing(asset_id))
}
