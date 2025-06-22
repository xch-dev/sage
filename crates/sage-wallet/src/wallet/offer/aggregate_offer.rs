use chia::{
    bls::Signature,
    protocol::{Bytes32, SpendBundle},
};

pub fn sort_offer(spend_bundle: SpendBundle) -> SpendBundle {
    let mut offered_coin_spends = Vec::new();
    let mut requested_coin_spends = Vec::new();

    for coin_spend in spend_bundle.coin_spends {
        if coin_spend.coin.parent_coin_info == Bytes32::default() {
            requested_coin_spends.push(coin_spend);
        } else {
            offered_coin_spends.push(coin_spend);
        }
    }

    SpendBundle::new(
        [requested_coin_spends, offered_coin_spends].concat(),
        spend_bundle.aggregated_signature,
    )
}

pub fn aggregate_offers(spend_bundles: Vec<SpendBundle>) -> SpendBundle {
    let mut aggregate = SpendBundle::new(Vec::new(), Signature::default());

    for spend_bundle in spend_bundles {
        aggregate.coin_spends.extend(spend_bundle.coin_spends);
        aggregate.aggregated_signature += &spend_bundle.aggregated_signature;
    }

    sort_offer(aggregate)
}
