use chia::{
    bls::Signature,
    protocol::{Bytes32, SpendBundle},
};
use chia_wallet_sdk::driver::Offer;

pub fn sort_offer(offer: Offer) -> Offer {
    let spend_bundle: SpendBundle = offer.into();

    let mut offered_coin_spends = Vec::new();
    let mut requested_coin_spends = Vec::new();

    for coin_spend in spend_bundle.coin_spends {
        if coin_spend.coin.parent_coin_info == Bytes32::default() {
            requested_coin_spends.push(coin_spend);
        } else {
            offered_coin_spends.push(coin_spend);
        }
    }

    Offer::new(SpendBundle::new(
        [requested_coin_spends, offered_coin_spends].concat(),
        spend_bundle.aggregated_signature,
    ))
}

pub fn aggregate_offers(offers: Vec<Offer>) -> Offer {
    let mut aggregate = SpendBundle::new(Vec::new(), Signature::default());

    for offer in offers {
        let spend_bundle: SpendBundle = offer.into();
        aggregate.coin_spends.extend(spend_bundle.coin_spends);
        aggregate.aggregated_signature += &spend_bundle.aggregated_signature;
    }

    sort_offer(Offer::new(aggregate))
}
