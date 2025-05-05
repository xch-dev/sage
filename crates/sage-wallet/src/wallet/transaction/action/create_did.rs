use std::mem;

use chia_wallet_sdk::driver::{HashedPtr, SpendContext};

use crate::{Action, Id, SingletonLineage, Spends, Summary, WalletError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreateDidAction;

impl Action for CreateDidAction {
    fn summarize(&self, summary: &mut Summary, index: usize) {
        summary.created_dids.insert(Id::New(index));
        summary.spent_xch += 1;
    }

    fn spend(
        &self,
        ctx: &mut SpendContext,
        spends: &mut Spends,
        index: usize,
    ) -> Result<(), WalletError> {
        let (item_ref, launcher) = spends.xch.create_launcher(ctx)?;
        let item = spends.xch.get_mut(item_ref)?;

        let (create_did, did) = launcher.create_simple_did(ctx, &item.p2)?;
        let did = did.with_metadata(HashedPtr::NIL);

        item.conditions = mem::take(&mut item.conditions).extend(create_did);

        spends.dids.insert(
            Id::New(index),
            SingletonLineage::new(did, item.p2, true, false),
        );

        Ok(())
    }
}
