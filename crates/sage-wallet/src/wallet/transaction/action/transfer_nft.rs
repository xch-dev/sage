use chia::protocol::Bytes32;
use chia_wallet_sdk::{driver::DidOwner, types::Conditions};

use crate::{Action, Id, Lineation, Summary, WalletError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransferNftAction {
    pub nft_id: Id,
    pub puzzle_hash: Bytes32,
    pub assign_did: AssignDid,
}

impl TransferNftAction {
    pub fn new(nft_id: Id, puzzle_hash: Bytes32, assign_did: AssignDid) -> Self {
        Self {
            nft_id,
            puzzle_hash,
            assign_did,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssignDid {
    Existing,
    None,
    New(Id),
}

impl Action for TransferNftAction {
    fn summarize(&self, summary: &mut Summary, _index: usize) {
        summary.spent_nfts.insert(self.nft_id);

        if let AssignDid::New(did_id) = self.assign_did {
            summary.spent_dids.insert(did_id);
        }
    }

    fn lineate(&self, lineation: &mut Lineation<'_>, _index: usize) -> Result<(), WalletError> {
        let nft = lineation.nfts[&self.nft_id];

        let nft_p2 = lineation
            .p2
            .get(&nft.info.p2_puzzle_hash)
            .ok_or(WalletError::MissingDerivation(nft.info.p2_puzzle_hash))?;

        let new_nft = match self.assign_did {
            AssignDid::Existing => {
                nft.transfer(lineation.ctx, nft_p2, self.puzzle_hash, Conditions::new())?
            }
            AssignDid::None => nft.transfer(
                lineation.ctx,
                nft_p2,
                self.puzzle_hash,
                Conditions::new().transfer_nft(None, Vec::new(), None),
            )?,
            AssignDid::New(did_id) => {
                let did = lineation.dids[&did_id];

                let did_p2 = lineation
                    .p2
                    .get(&did.info.p2_puzzle_hash)
                    .ok_or(WalletError::MissingDerivation(did.info.p2_puzzle_hash))?;

                let (conditions, nft) = nft.transfer_to_did(
                    lineation.ctx,
                    nft_p2,
                    self.puzzle_hash,
                    Some(DidOwner::from_did_info(&did.info)),
                    Conditions::new(),
                )?;

                lineation.dids[&did_id] = did.update(lineation.ctx, did_p2, conditions)?;

                nft
            }
        };

        lineation.nfts[&self.nft_id] = new_nft;

        Ok(())
    }
}
