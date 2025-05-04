use chia_wallet_sdk::driver::HashedPtr;

use crate::{Action, Distribution, Id, Summary, WalletError};

/// This will create a new DID at the change puzzle hash specified
/// in the transaction config. It can be immediately spent if needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CreateDidAction;

impl Action for CreateDidAction {
    fn summarize(&self, summary: &mut Summary, index: usize) {
        summary.created_dids.insert(Id::New(index));
        summary.spent_xch += 1;
    }

    fn distribute(
        &self,
        distribution: &mut Distribution<'_>,
        index: usize,
    ) -> Result<(), WalletError> {
        if distribution.asset_id().is_some() {
            return Ok(());
        }

        distribution.create_launcher(|ctx, new_assets, item, launcher, conditions| {
            let (create_did, did) = launcher.create_simple_did(ctx, &item.p2)?;

            new_assets
                .dids
                .insert(Id::New(index), did.with_metadata(HashedPtr::NIL));

            Ok(conditions.extend(create_did))
        })
    }
}
