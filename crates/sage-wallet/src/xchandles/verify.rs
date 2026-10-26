use chia_wallet_sdk::{
    chia::puzzle_types::{LineageProof, singleton::SingletonArgs},
    driver::Slot,
    prelude::*,
    types::puzzles::{SlotInfo, XchandlesHandleSlotValue, XchandlesSlotNonce},
};

use crate::{ChildKind, Handle, HandleResponse, SingletonResponse, WalletPeer, XchandlesError};

/// The registry singleton always has an amount of 1, so neither does the parent of a slot.
const REGISTRY_AMOUNT: u64 = 1;

/// How many spends of the resolved singleton to follow past the coin the API reported,
/// in case the API hasn't indexed its latest transfers yet.
const MAX_SINGLETON_HOPS: usize = 8;

/// The peak transaction block, which on-chain time locks are evaluated against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChainTip {
    pub height: u32,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedSingletonKind {
    Nft,
    Did,
}

/// A handle resolution whose slot and resolved singleton were checked against the chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedHandle {
    pub handle: Handle,
    pub registry_launcher_id: Bytes32,
    pub slot_value: XchandlesHandleSlotValue,
    pub slot_proof: LineageProof,
    pub slot_parent_id: Bytes32,
    pub slot_coin_id: Bytes32,
    pub slot_confirmation_height: u32,
    pub resolved_kind: ResolvedSingletonKind,
    pub resolved_coin_id: Bytes32,
    pub resolved_confirmation_height: u32,
    /// The owner puzzle hash of the resolved singleton, which is the address the handle
    /// resolves to.
    pub resolved_puzzle_hash: Bytes32,
    pub indexed_height: u32,
    pub tip: ChainTip,
}

impl VerifiedHandle {
    pub fn handle_hash(&self) -> Bytes32 {
        self.slot_value.handle_hash
    }

    pub fn owner_launcher_id(&self) -> Bytes32 {
        self.slot_value.owner_launcher_id
    }

    pub fn resolved_launcher_id(&self) -> Bytes32 {
        self.slot_value.resolved_launcher_id
    }

    pub fn expiration(&self) -> u64 {
        self.slot_value.expiration
    }
}

/// Rebuilds the handle slot described by an API response, checking that its parent is a
/// coin of the trusted registry singleton.
///
/// A slot's puzzle hash alone can be created by anyone. What makes a slot authentic is
/// that its parent is the registry singleton, and a coin with the registry's singleton
/// puzzle hash can only have been spent if its own lineage leads back to the launcher.
pub fn handle_slot(
    registry_launcher_id: Bytes32,
    response: &HandleResponse,
) -> Result<Slot<XchandlesHandleSlotValue>, XchandlesError> {
    let slot = response.slot;

    let value = XchandlesHandleSlotValue::new(
        slot.counter,
        slot.handle_hash,
        slot.neighbors.left_value,
        slot.neighbors.right_value,
        slot.expiration,
        slot.owner_launcher_id,
        slot.resolved_launcher_id,
    );

    let proof = LineageProof {
        parent_parent_coin_info: response.slot_parent_parent_id,
        parent_inner_puzzle_hash: response.slot_parent_inner_puzzle_hash,
        parent_amount: REGISTRY_AMOUNT,
    };

    let slot = Slot::new(
        proof,
        SlotInfo::from_value(
            registry_launcher_id,
            XchandlesSlotNonce::HANDLE.to_u64(),
            value,
        ),
    );

    if slot.coin.parent_coin_info != response.slot_parent_id {
        return Err(XchandlesError::InvalidLineage);
    }

    Ok(slot)
}

/// Verifies an API response for a handle against the chain.
pub async fn verify_handle(
    peer: &WalletPeer,
    genesis_challenge: Bytes32,
    registry_launcher_id: Bytes32,
    handle: &Handle,
    response: &HandleResponse,
    tip: ChainTip,
) -> Result<VerifiedHandle, XchandlesError> {
    if response.registry_launcher_id != registry_launcher_id
        || response.handle != handle.as_str()
        || response.slot.handle_hash != handle.hash()
    {
        return Err(XchandlesError::ResponseMismatch);
    }

    let slot = handle_slot(registry_launcher_id, response)?;
    let slot_coin_id = slot.coin.coin_id();

    let slot_state = peer
        .fetch_optional_coin(slot_coin_id, genesis_challenge)
        .await?;

    let Some(slot_confirmation_height) = slot_state.and_then(|state| state.created_height) else {
        return Err(XchandlesError::SlotNotFound(slot_coin_id));
    };

    // Slots are recreated whenever a neighboring handle changes, so a spent slot most
    // likely means the API is behind, not that the handle has changed.
    if slot_state.is_some_and(|state| state.spent_height.is_some()) {
        return Err(XchandlesError::SlotSpent(slot_coin_id));
    }

    // The expire action can be performed once the timestamp reaches the expiration. This
    // is only checked once the slot is confirmed, since until then the expiration is just
    // the API's claim.
    if slot.info.value.expiration <= tip.timestamp {
        return Err(XchandlesError::Expired {
            expiration: slot.info.value.expiration,
            timestamp: tip.timestamp,
        });
    }

    let resolved_launcher_id = slot.info.value.resolved_launcher_id;

    let singleton = response
        .resolved_singleton
        .as_ref()
        .ok_or(XchandlesError::ResolutionUnavailable)?;

    let resolved =
        verify_resolved_singleton(peer, genesis_challenge, resolved_launcher_id, singleton).await?;

    Ok(VerifiedHandle {
        handle: handle.clone(),
        registry_launcher_id,
        slot_value: slot.info.value,
        slot_proof: slot.proof,
        slot_parent_id: slot.coin.parent_coin_info,
        slot_coin_id,
        slot_confirmation_height,
        resolved_kind: resolved.kind,
        resolved_coin_id: resolved.coin.coin_id(),
        resolved_confirmation_height: resolved.confirmation_height,
        resolved_puzzle_hash: resolved.p2_puzzle_hash,
        indexed_height: response.indexed_peak_height,
        tip,
    })
}

struct ResolvedSingleton {
    kind: ResolvedSingletonKind,
    coin: Coin,
    confirmation_height: u32,
    p2_puzzle_hash: Bytes32,
}

/// Finds the latest unspent coin of the resolved singleton, starting at the coin the API
/// reported, and parses its owner from the spend that created it.
async fn verify_resolved_singleton(
    peer: &WalletPeer,
    genesis_challenge: Bytes32,
    launcher_id: Bytes32,
    singleton: &SingletonResponse,
) -> Result<ResolvedSingleton, XchandlesError> {
    if singleton.launcher_id != launcher_id {
        return Err(XchandlesError::ResponseMismatch);
    }

    // The API's melted flag isn't trusted. A melted singleton is detected below from its
    // last coin being spent without creating a successor.

    let mut coin = Coin::new(
        singleton.parent_coin_id,
        SingletonArgs::curry_tree_hash(launcher_id, singleton.inner_puzzle_hash.into()).into(),
        singleton.amount,
    );

    for _ in 0..MAX_SINGLETON_HOPS {
        let coin_id = coin.coin_id();

        // The eve coin is created by the launcher, whose spend doesn't reveal the owner. If
        // it's already been spent, its child can be parsed instead.
        if coin.parent_coin_info == launcher_id {
            let Some(coin_state) = peer.fetch_optional_coin(coin_id, genesis_challenge).await?
            else {
                return Err(XchandlesError::ResolvedSingletonNotFound(coin_id));
            };

            if coin_state.spent_height.is_none() {
                return Err(XchandlesError::ResolvedSingletonUnspentEve(launcher_id));
            }

            let Some(child) = peer.try_fetch_singleton_child(coin_id).await? else {
                return Err(XchandlesError::ResolvedSingletonMelted(launcher_id));
            };

            coin = child.coin;
            continue;
        }

        let Some(parent_state) = peer
            .fetch_optional_coin(coin.parent_coin_info, genesis_challenge)
            .await?
        else {
            return Err(XchandlesError::ResolvedSingletonNotFound(coin_id));
        };

        let Some(parent_spent_height) = parent_state.spent_height else {
            return Err(XchandlesError::ResolvedSingletonNotFound(coin_id));
        };

        let (parent_puzzle, parent_solution) = peer
            .fetch_puzzle_solution(coin.parent_coin_info, parent_spent_height)
            .await?;

        let (kind, p2_puzzle_hash) = match ChildKind::from_parent(
            parent_state.coin,
            &parent_puzzle,
            &parent_solution,
            coin,
        )? {
            ChildKind::Nft { info, clawback, .. } if info.launcher_id == launcher_id => {
                if clawback.is_some() {
                    return Err(XchandlesError::ResolvedSingletonInClawback(launcher_id));
                }
                (ResolvedSingletonKind::Nft, info.p2_puzzle_hash)
            }
            ChildKind::Did { info, clawback, .. } if info.launcher_id == launcher_id => {
                if clawback.is_some() {
                    return Err(XchandlesError::ResolvedSingletonInClawback(launcher_id));
                }
                (ResolvedSingletonKind::Did, info.p2_puzzle_hash)
            }
            _ => return Err(XchandlesError::InvalidResolvedSingleton(coin_id)),
        };

        let Some(coin_state) = peer.fetch_optional_coin(coin_id, genesis_challenge).await? else {
            return Err(XchandlesError::ResolvedSingletonNotFound(coin_id));
        };

        let Some(confirmation_height) = coin_state.created_height else {
            return Err(XchandlesError::ResolvedSingletonNotFound(coin_id));
        };

        if coin_state.spent_height.is_none() {
            return Ok(ResolvedSingleton {
                kind,
                coin,
                confirmation_height,
                p2_puzzle_hash,
            });
        }

        // The singleton has moved since the API indexed it. Its child is parsed from this
        // coin's spend on the next iteration, so nothing about it is taken on trust.
        let Some(child) = peer.try_fetch_singleton_child(coin_id).await? else {
            return Err(XchandlesError::ResolvedSingletonMelted(launcher_id));
        };

        coin = child.coin;
    }

    Err(XchandlesError::ResolvedSingletonNotFound(coin.coin_id()))
}
