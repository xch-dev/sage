use chia_wallet_sdk::{
    chia::puzzle_types::LineageProof,
    driver::{SingletonLayer, Slot},
    prelude::*,
    types::puzzles::{SlotInfo, XchandlesSlotNonce, XchandlesUpdateSlotValue},
};

use crate::{Handle, PendingTransferResponse, WalletPeer, XchandlesError};

/// The registry slot a pending transfer is recorded in, if the API's description of it is
/// accurate.
pub fn update_slot_info(
    registry_launcher_id: Bytes32,
    handle: &Handle,
    transfer: &PendingTransferResponse,
) -> SlotInfo<XchandlesUpdateSlotValue> {
    SlotInfo::from_value(
        registry_launcher_id,
        XchandlesSlotNonce::UPDATE.to_u64(),
        XchandlesUpdateSlotValue::new(
            transfer.initiator_coin_id,
            transfer.minimum_execution_height,
            handle.hash(),
            transfer.new_owner_launcher_id,
            transfer.new_resolved_launcher_id,
        ),
    )
}

/// Checks a pending transfer reported by the API against the chain, returning the update
/// slot coin which records it.
///
/// Initiating an update creates an update slot holding the new owner and resolved launcher
/// IDs, which is spent when the update is executed. The transfer is only verified if an
/// unspent coin with exactly that slot's puzzle hash exists and was created by the trusted
/// registry singleton, since anyone can create a coin with a slot puzzle hash. `None` means
/// the API's claim couldn't be confirmed, for example because the update has already been
/// executed or the claim is false.
///
/// The API is the only source of pending transfers, so a transfer it doesn't report can't be
/// detected.
pub async fn verify_pending_transfer(
    peer: &WalletPeer,
    genesis_challenge: Bytes32,
    registry_launcher_id: Bytes32,
    handle: &Handle,
    transfer: &PendingTransferResponse,
) -> Result<Option<Bytes32>, XchandlesError> {
    let info = update_slot_info(registry_launcher_id, handle, transfer);
    let puzzle_hash = Slot::puzzle_hash(&info).into();

    for state in peer
        .fetch_unspent_coins_by_puzzle_hash(puzzle_hash, genesis_challenge)
        .await?
    {
        if state.created_height.is_none() {
            continue;
        }

        let Some(parent) = peer
            .fetch_optional_coin(state.coin.parent_coin_info, genesis_challenge)
            .await?
        else {
            continue;
        };

        let Some(parent_spent_height) = parent.spent_height else {
            continue;
        };

        let (parent_puzzle, _) = peer
            .fetch_puzzle_solution(state.coin.parent_coin_info, parent_spent_height)
            .await?;

        let Some(parent_inner_puzzle_hash) =
            registry_inner_puzzle_hash(&parent_puzzle, registry_launcher_id)
        else {
            continue;
        };

        // Rebuilding the slot from its parent's lineage proves the parent had the registry's
        // singleton puzzle hash, which only the registry itself can spend.
        let slot = Slot::new(
            LineageProof {
                parent_parent_coin_info: parent.coin.parent_coin_info,
                parent_inner_puzzle_hash,
                parent_amount: parent.coin.amount,
            },
            update_slot_info(registry_launcher_id, handle, transfer),
        );

        if slot.coin == state.coin {
            return Ok(Some(slot.coin.coin_id()));
        }
    }

    Ok(None)
}

/// The inner puzzle hash of a spent coin of the registry singleton, given its puzzle reveal.
fn registry_inner_puzzle_hash(puzzle: &Program, registry_launcher_id: Bytes32) -> Option<Bytes32> {
    let mut allocator = Allocator::new();
    let ptr = puzzle.to_clvm(&mut allocator).ok()?;
    let puzzle = Puzzle::parse(&allocator, ptr);
    let singleton = SingletonLayer::<Puzzle>::parse_puzzle(&allocator, puzzle).ok()??;

    (singleton.launcher_id == registry_launcher_id)
        .then(|| singleton.inner_puzzle.curried_puzzle_hash().into())
}
