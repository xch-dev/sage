use chia_wallet_sdk::driver::Offer;

use crate::{Select, Selection, WalletError};

/// Takes an offer and makes the assets received from the offer available
/// to be spent as part of this transaction (or sent to your change address).
#[derive(Debug, Clone)]
pub struct TakeOfferAction {
    pub offer: Offer,
}

impl Select for TakeOfferAction {
    fn select(&self, selection: &mut Selection, _index: usize) -> Result<(), WalletError> {
        Ok(())
    }
}
