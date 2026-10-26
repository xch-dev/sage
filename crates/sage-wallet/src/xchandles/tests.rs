use std::{collections::VecDeque, sync::Mutex};

use chia_wallet_sdk::{
    chia::puzzle_types::{
        EveProof, LineageProof, Proof,
        singleton::{SingletonArgs, SingletonSolution},
    },
    clvm_utils::tree_hash_atom,
    driver::Slot,
    prelude::*,
    test::{BlsPairWithCoin, PeerSimulator},
    types::puzzles::{
        SlotInfo, XchandlesHandleSlotValue, XchandlesSlotNonce, XchandlesUpdateSlotValue,
    },
};

use crate::WalletPeer;

use super::*;

const FAR_FUTURE: u64 = 4_000_000_000;

fn genesis_challenge() -> Bytes32 {
    TESTNET11_CONSTANTS.genesis_challenge
}

fn tip(timestamp: u64) -> ChainTip {
    ChainTip {
        height: 10,
        timestamp,
    }
}

/// A singleton standing in for the registry. Its inner puzzle returns its solution as
/// conditions, so it can create and spend slots exactly like the real registry does.
struct TestRegistry {
    launcher_id: Bytes32,
    coin: Coin,
    proof: Proof,
}

impl TestRegistry {
    fn inner_puzzle_hash() -> Bytes32 {
        tree_hash_atom(&[1]).into()
    }

    async fn launch(sim: &PeerSimulator) -> anyhow::Result<Self> {
        let funding = sim.lock().await.bls(1);
        let mut ctx = SpendContext::new();

        let launcher = Launcher::new(funding.coin.coin_id(), 1);
        let launcher_id = launcher.coin().coin_id();
        let (conditions, eve) = launcher.spend(&mut ctx, Self::inner_puzzle_hash(), ())?;
        StandardLayer::new(funding.pk).spend(&mut ctx, funding.coin, conditions)?;
        sim.lock()
            .await
            .spend_coins(ctx.take(), std::slice::from_ref(&funding.sk))?;

        Ok(Self {
            launcher_id,
            coin: eve,
            proof: Proof::Eve(EveProof {
                parent_parent_coin_info: funding.coin.coin_id(),
                parent_amount: 1,
            }),
        })
    }

    /// Spends the registry, spending and creating the given handle slots.
    async fn spend(
        &mut self,
        sim: &PeerSimulator,
        spend: &[Slot<XchandlesHandleSlotValue>],
        create: &[XchandlesHandleSlotValue],
    ) -> anyhow::Result<Vec<Slot<XchandlesHandleSlotValue>>> {
        let mut ctx = SpendContext::new();
        let inner_puzzle_hash = Self::inner_puzzle_hash();

        // The singleton layer wraps the odd output in the singleton puzzle.
        let mut conditions = Conditions::new().create_coin(inner_puzzle_hash, 1, Memos::None);

        for slot in spend {
            let slot_puzzle_hash = ctx.alloc(&slot.coin.puzzle_hash)?;
            conditions = conditions.send_message(18, Bytes::default(), vec![slot_puzzle_hash]);
            slot.clone().spend(&mut ctx, inner_puzzle_hash)?;
        }

        let mut created = Vec::new();

        for &value in create {
            let slot = Slot::new(
                LineageProof {
                    parent_parent_coin_info: self.coin.parent_coin_info,
                    parent_inner_puzzle_hash: inner_puzzle_hash,
                    parent_amount: 1,
                },
                SlotInfo::from_value(self.launcher_id, XchandlesSlotNonce::HANDLE.to_u64(), value),
            );
            assert_eq!(slot.coin.parent_coin_info, self.coin.coin_id());
            conditions = conditions.create_coin(slot.coin.puzzle_hash, 0, Memos::None);
            created.push(slot);
        }

        self.finish_spend(sim, ctx, conditions).await?;

        Ok(created)
    }

    /// Spends the registry, creating an update slot as the initiate update action does.
    async fn create_update_slot(
        &mut self,
        sim: &PeerSimulator,
        info: SlotInfo<XchandlesUpdateSlotValue>,
    ) -> anyhow::Result<Slot<XchandlesUpdateSlotValue>> {
        let inner_puzzle_hash = Self::inner_puzzle_hash();

        let slot = Slot::new(
            LineageProof {
                parent_parent_coin_info: self.coin.parent_coin_info,
                parent_inner_puzzle_hash: inner_puzzle_hash,
                parent_amount: 1,
            },
            info,
        );

        let conditions = Conditions::new()
            .create_coin(inner_puzzle_hash, 1, Memos::None)
            .create_coin(slot.coin.puzzle_hash, 0, Memos::None);

        self.finish_spend(sim, SpendContext::new(), conditions)
            .await?;

        Ok(slot)
    }

    async fn finish_spend(
        &mut self,
        sim: &PeerSimulator,
        mut ctx: SpendContext,
        conditions: Conditions,
    ) -> anyhow::Result<()> {
        let inner_puzzle_hash = Self::inner_puzzle_hash();
        let inner_puzzle = ctx.alloc(&1)?;
        let puzzle = ctx.curry(SingletonArgs::new(self.launcher_id, inner_puzzle))?;
        let solution = ctx.alloc(&SingletonSolution {
            lineage_proof: self.proof,
            amount: 1,
            inner_solution: conditions,
        })?;
        ctx.spend(self.coin, Spend::new(puzzle, solution))?;

        sim.lock().await.spend_coins(ctx.take(), &[])?;

        self.proof = Proof::Lineage(LineageProof {
            parent_parent_coin_info: self.coin.parent_coin_info,
            parent_inner_puzzle_hash: inner_puzzle_hash,
            parent_amount: 1,
        });
        self.coin = Coin::new(self.coin.coin_id(), self.coin.puzzle_hash, 1);

        Ok(())
    }
}

async fn mint_nft(sim: &PeerSimulator) -> anyhow::Result<(Nft, BlsPairWithCoin)> {
    let owner = sim.lock().await.bls(1);
    let mut ctx = SpendContext::new();

    let (conditions, nft) = Launcher::new(owner.coin.coin_id(), 1).mint_nft(
        &mut ctx,
        &NftMint::new(HashedPtr::NIL, owner.puzzle_hash, 0, None),
    )?;
    StandardLayer::new(owner.pk).spend(&mut ctx, owner.coin, conditions)?;
    sim.lock()
        .await
        .spend_coins(ctx.take(), std::slice::from_ref(&owner.sk))?;

    Ok((nft, owner))
}

async fn transfer_nft(
    sim: &PeerSimulator,
    nft: &Nft,
    owner: &BlsPairWithCoin,
    puzzle_hash: Bytes32,
) -> anyhow::Result<Nft> {
    let mut ctx = SpendContext::new();
    let nft = (*nft).transfer(
        &mut ctx,
        &StandardLayer::new(owner.pk),
        puzzle_hash,
        Conditions::new(),
    )?;
    sim.lock()
        .await
        .spend_coins(ctx.take(), std::slice::from_ref(&owner.sk))?;
    Ok(nft)
}

fn slot_value(
    handle: &Handle,
    resolved_launcher_id: Bytes32,
    left: u8,
) -> XchandlesHandleSlotValue {
    XchandlesHandleSlotValue::new(
        0,
        handle.hash(),
        Bytes32::new([left; 32]),
        Bytes32::new([0xff; 32]),
        FAR_FUTURE,
        resolved_launcher_id,
        resolved_launcher_id,
    )
}

fn response(
    registry_launcher_id: Bytes32,
    handle: &Handle,
    slot: &Slot<XchandlesHandleSlotValue>,
    nft: &Nft,
) -> HandleResponse {
    let value = slot.info.value;

    HandleResponse {
        registry_launcher_id,
        handle: handle.as_str().to_string(),
        slot: HandleSlotResponse {
            counter: value.counter,
            handle_hash: value.handle_hash,
            neighbors: SlotNeighborsResponse {
                left_value: value.neighbors.left_value,
                right_value: value.neighbors.right_value,
            },
            expiration: value.expiration,
            owner_launcher_id: value.owner_launcher_id,
            resolved_launcher_id: value.resolved_launcher_id,
        },
        slot_parent_id: slot.coin.parent_coin_info,
        slot_parent_parent_id: slot.proof.parent_parent_coin_info,
        slot_parent_inner_puzzle_hash: slot.proof.parent_inner_puzzle_hash,
        slot_confirmation_height: 1,
        resolved_singleton: Some(SingletonResponse {
            launcher_id: nft.info.launcher_id,
            parent_coin_id: nft.coin.parent_coin_info,
            amount: nft.coin.amount,
            inner_puzzle_hash: nft.info.inner_puzzle_hash().into(),
            confirmation_height: 1,
            melted: false,
            melt_height: None,
            nft: None,
            indexed_peak_height: 1,
        }),
        indexed_peak_height: 1,
        pending_transfer: None,
    }
}

struct Setup {
    sim: PeerSimulator,
    peer: WalletPeer,
    registry: TestRegistry,
    handle: Handle,
    slot: Slot<XchandlesHandleSlotValue>,
    nft: Nft,
    owner: BlsPairWithCoin,
}

impl Setup {
    async fn new() -> anyhow::Result<Self> {
        let sim = PeerSimulator::new().await?;
        let peer = WalletPeer::new(sim.connect().await?);
        let mut registry = TestRegistry::launch(&sim).await?;
        let (nft, owner) = mint_nft(&sim).await?;

        let handle = Handle::parse("alice")?;
        let slot = registry
            .spend(&sim, &[], &[slot_value(&handle, nft.info.launcher_id, 0)])
            .await?
            .remove(0);

        Ok(Self {
            sim,
            peer,
            registry,
            handle,
            slot,
            nft,
            owner,
        })
    }

    fn response(&self) -> HandleResponse {
        response(
            self.registry.launcher_id,
            &self.handle,
            &self.slot,
            &self.nft,
        )
    }

    async fn verify(&self, response: &HandleResponse) -> Result<VerifiedHandle, XchandlesError> {
        verify_handle(
            &self.peer,
            genesis_challenge(),
            self.registry.launcher_id,
            &self.handle,
            response,
            tip(1000),
        )
        .await
    }
}

struct MockIndex(Mutex<VecDeque<HandleResponse>>);

impl MockIndex {
    fn new(responses: impl IntoIterator<Item = HandleResponse>) -> Self {
        Self(Mutex::new(responses.into_iter().collect()))
    }

    fn remaining(&self) -> usize {
        self.0.lock().unwrap().len()
    }
}

impl XchandlesIndex for MockIndex {
    async fn handle(&self, _handle: &Handle) -> Result<HandleResponse, XchandlesApiError> {
        self.0
            .lock()
            .unwrap()
            .pop_front()
            .ok_or(XchandlesApiError::IndexStale)
    }
}

#[tokio::test]
async fn test_verify_handle() -> anyhow::Result<()> {
    let setup = Setup::new().await?;

    let verified = setup.verify(&setup.response()).await?;

    assert_eq!(verified.handle, setup.handle);
    assert_eq!(verified.handle_hash(), setup.handle.hash());
    assert_eq!(verified.slot_coin_id, setup.slot.coin.coin_id());
    assert_eq!(verified.resolved_launcher_id(), setup.nft.info.launcher_id);
    assert_eq!(verified.resolved_kind, ResolvedSingletonKind::Nft);
    assert_eq!(verified.resolved_coin_id, setup.nft.coin.coin_id());
    assert_eq!(verified.resolved_puzzle_hash, setup.owner.puzzle_hash);
    assert_eq!(verified.expiration(), FAR_FUTURE);

    Ok(())
}

#[tokio::test]
async fn test_forged_slot_lineage_is_rejected() -> anyhow::Result<()> {
    let setup = Setup::new().await?;

    // Anyone can create a coin with an authentic slot puzzle hash. Here a standard coin
    // creates one, rather than the registry.
    let forger = setup.sim.lock().await.bls(1);
    let mut ctx = SpendContext::new();
    StandardLayer::new(forger.pk).spend(
        &mut ctx,
        forger.coin,
        Conditions::new().create_coin(setup.slot.coin.puzzle_hash, 0, Memos::None),
    )?;
    setup
        .sim
        .lock()
        .await
        .spend_coins(ctx.take(), std::slice::from_ref(&forger.sk))?;

    let mut response = setup.response();
    response.slot_parent_id = forger.coin.coin_id();
    response.slot_parent_parent_id = forger.coin.parent_coin_info;
    response.slot_parent_inner_puzzle_hash = forger.puzzle_hash;

    assert!(matches!(
        setup.verify(&response).await,
        Err(XchandlesError::InvalidLineage)
    ));

    Ok(())
}

#[tokio::test]
async fn test_tampered_slot_value_is_not_verified() -> anyhow::Result<()> {
    let setup = Setup::new().await?;

    // The lineage is real, but the slot value now points at a different singleton. No
    // such slot exists on chain.
    let mut response = setup.response();
    response.slot.resolved_launcher_id = Bytes32::new([9; 32]);

    let error = setup.verify(&response).await.unwrap_err();
    assert!(matches!(error, XchandlesError::SlotNotFound(..)));

    Ok(())
}

#[tokio::test]
async fn test_unconfirmed_expiration_is_not_trusted() -> anyhow::Result<()> {
    let setup = Setup::new().await?;

    // An API claiming the handle has expired isn't believed until the slot carrying that
    // expiration is found on chain.
    let mut response = setup.response();
    response.slot.expiration = 500;

    let error = setup.verify(&response).await.unwrap_err();
    assert!(matches!(error, XchandlesError::SlotNotFound(..)));
    assert!(!error.is_rejection());

    Ok(())
}

#[tokio::test]
async fn test_slot_from_another_registry_is_rejected() -> anyhow::Result<()> {
    let setup = Setup::new().await?;

    // A second deployment with an identical slot for the same handle.
    let mut other = TestRegistry::launch(&setup.sim).await?;
    let other_slot = other
        .spend(
            &setup.sim,
            &[],
            &[slot_value(&setup.handle, setup.nft.info.launcher_id, 0)],
        )
        .await?
        .remove(0);

    // Honestly labeled, it's for the wrong registry.
    let honest = response(other.launcher_id, &setup.handle, &other_slot, &setup.nft);
    assert!(matches!(
        setup.verify(&honest).await,
        Err(XchandlesError::ResponseMismatch)
    ));

    // Relabeled as the trusted registry, the lineage doesn't match.
    let relabeled = response(
        setup.registry.launcher_id,
        &setup.handle,
        &other_slot,
        &setup.nft,
    );
    assert!(matches!(
        setup.verify(&relabeled).await,
        Err(XchandlesError::InvalidLineage)
    ));

    // Verifying with the wrong deployment constants rejects the authentic response.
    let error = verify_handle(
        &setup.peer,
        genesis_challenge(),
        other.launcher_id,
        &setup.handle,
        &setup.response(),
        tip(1000),
    )
    .await
    .unwrap_err();
    assert!(matches!(error, XchandlesError::ResponseMismatch));

    Ok(())
}

#[tokio::test]
async fn test_response_for_other_handle_is_rejected() -> anyhow::Result<()> {
    let setup = Setup::new().await?;

    let error = verify_handle(
        &setup.peer,
        genesis_challenge(),
        setup.registry.launcher_id,
        &Handle::parse("mallory")?,
        &setup.response(),
        tip(1000),
    )
    .await
    .unwrap_err();
    assert!(matches!(error, XchandlesError::ResponseMismatch));

    Ok(())
}

#[tokio::test]
async fn test_expired_handle_is_rejected() -> anyhow::Result<()> {
    let setup = Setup::new().await?;

    for timestamp in [FAR_FUTURE, FAR_FUTURE + 1] {
        let error = verify_handle(
            &setup.peer,
            genesis_challenge(),
            setup.registry.launcher_id,
            &setup.handle,
            &setup.response(),
            tip(timestamp),
        )
        .await
        .unwrap_err();
        assert!(matches!(error, XchandlesError::Expired { .. }));
        assert!(error.is_not_registered());
    }

    Ok(())
}

#[tokio::test]
async fn test_spent_slot_is_fetched_again() -> anyhow::Result<()> {
    let mut setup = Setup::new().await?;
    let stale = setup.response();

    // A neighboring handle is registered, so the slot is recreated with new neighbors but
    // the same mapping.
    let neighbor_value = slot_value(&setup.handle, setup.nft.info.launcher_id, 1);
    let recreated = setup
        .registry
        .spend(
            &setup.sim,
            std::slice::from_ref(&setup.slot),
            &[neighbor_value],
        )
        .await?
        .remove(0);
    let old_slot = std::mem::replace(&mut setup.slot, recreated);
    let fresh = setup.response();

    let error = setup.verify(&stale).await.unwrap_err();
    assert!(matches!(error, XchandlesError::SlotSpent(id) if id == old_slot.coin.coin_id()));
    assert!(error.is_unavailable());

    let index = MockIndex::new([stale, fresh]);
    let verified = resolve_handle(
        &index,
        &setup.peer,
        genesis_challenge(),
        setup.registry.launcher_id,
        &setup.handle,
        async { Ok(tip(1000)) },
    )
    .await?;

    assert_eq!(index.remaining(), 0);
    assert_eq!(verified.slot_coin_id, setup.slot.coin.coin_id());
    assert_eq!(verified.resolved_puzzle_hash, setup.owner.puzzle_hash);

    Ok(())
}

#[tokio::test]
async fn test_resolve_gives_up_on_persistently_spent_slot() -> anyhow::Result<()> {
    let mut setup = Setup::new().await?;
    let stale = setup.response();

    let neighbor_value = slot_value(&setup.handle, setup.nft.info.launcher_id, 1);
    setup
        .registry
        .spend(
            &setup.sim,
            std::slice::from_ref(&setup.slot),
            &[neighbor_value],
        )
        .await?;

    let index = MockIndex::new(std::iter::repeat_n(stale, MAX_RESOLVE_ATTEMPTS + 1));
    let error = resolve_handle(
        &index,
        &setup.peer,
        genesis_challenge(),
        setup.registry.launcher_id,
        &setup.handle,
        async { Ok(tip(1000)) },
    )
    .await
    .unwrap_err();

    assert!(matches!(error, XchandlesError::SlotSpent(..)));
    assert_eq!(index.remaining(), 1);

    Ok(())
}

#[tokio::test]
async fn test_resolved_singleton_transfers_are_followed() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let response = setup.response();

    // The NFT moves twice after the API indexed it.
    let middle = setup.sim.lock().await.bls(0);
    let nft = transfer_nft(&setup.sim, &setup.nft, &setup.owner, middle.puzzle_hash).await?;
    let nft = transfer_nft(&setup.sim, &nft, &middle, Bytes32::new([5; 32])).await?;

    let verified = setup.verify(&response).await?;
    assert_eq!(verified.resolved_coin_id, nft.coin.coin_id());
    assert_eq!(verified.resolved_puzzle_hash, Bytes32::new([5; 32]));

    Ok(())
}

#[tokio::test]
async fn test_too_many_transfers_are_not_verified() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let response = setup.response();

    // The NFT moves as many times as can be followed past the coin the API reported.
    let mut nft = setup.nft;
    for _ in 0..MAX_SINGLETON_HOPS {
        nft = transfer_nft(&setup.sim, &nft, &setup.owner, setup.owner.puzzle_hash).await?;
    }

    let error = setup.verify(&response).await.unwrap_err();
    assert!(matches!(
        error,
        XchandlesError::ResolvedSingletonMovedTooOften(..)
    ));
    assert!(!error.is_unavailable() && error.is_rejection());

    Ok(())
}

#[tokio::test]
async fn test_invalid_resolved_singleton_is_rejected() -> anyhow::Result<()> {
    let setup = Setup::new().await?;

    let mut response = setup.response();
    response.resolved_singleton = None;
    assert!(matches!(
        setup.verify(&response).await,
        Err(XchandlesError::ResolutionUnavailable)
    ));

    let mut response = setup.response();
    response.resolved_singleton.as_mut().unwrap().launcher_id = Bytes32::new([9; 32]);
    assert!(matches!(
        setup.verify(&response).await,
        Err(XchandlesError::ResponseMismatch)
    ));

    // A coin which the parent never created.
    let mut response = setup.response();
    response
        .resolved_singleton
        .as_mut()
        .unwrap()
        .inner_puzzle_hash = Bytes32::new([9; 32]);
    assert!(matches!(
        setup.verify(&response).await,
        Err(XchandlesError::InvalidResolvedSingleton(..))
    ));

    // A coin whose parent doesn't exist.
    let mut response = setup.response();
    response.resolved_singleton.as_mut().unwrap().parent_coin_id = Bytes32::new([9; 32]);
    assert!(matches!(
        setup.verify(&response).await,
        Err(XchandlesError::ResolvedSingletonNotFound(..))
    ));

    Ok(())
}

#[tokio::test]
async fn test_melted_flag_is_not_trusted() -> anyhow::Result<()> {
    let setup = Setup::new().await?;

    let mut response = setup.response();
    response.resolved_singleton.as_mut().unwrap().melted = true;
    let verified = setup.verify(&response).await?;
    assert_eq!(verified.resolved_coin_id, setup.nft.coin.coin_id());

    Ok(())
}

#[tokio::test]
async fn test_spent_eve_singleton_is_followed() -> anyhow::Result<()> {
    let setup = Setup::new().await?;

    let Proof::Lineage(proof) = setup.nft.proof else {
        panic!("expected the minted NFT to have spent its eve coin");
    };

    let mut response = setup.response();
    let singleton = response.resolved_singleton.as_mut().unwrap();
    singleton.parent_coin_id = setup.nft.info.launcher_id;
    singleton.inner_puzzle_hash = proof.parent_inner_puzzle_hash;

    let verified = setup.verify(&response).await?;
    assert_eq!(verified.resolved_coin_id, setup.nft.coin.coin_id());
    assert_eq!(verified.resolved_puzzle_hash, setup.owner.puzzle_hash);

    Ok(())
}

#[tokio::test]
async fn test_unspent_eve_singleton_is_not_verified() -> anyhow::Result<()> {
    let mut setup = Setup::new().await?;

    // A singleton which was launched but never spent, so nothing reveals its owner.
    let unspent = TestRegistry::launch(&setup.sim).await?;
    setup.slot = setup
        .registry
        .spend(
            &setup.sim,
            std::slice::from_ref(&setup.slot),
            &[slot_value(&setup.handle, unspent.launcher_id, 1)],
        )
        .await?
        .remove(0);

    let mut response = setup.response();
    let singleton = response.resolved_singleton.as_mut().unwrap();
    singleton.launcher_id = unspent.launcher_id;
    singleton.parent_coin_id = unspent.launcher_id;
    singleton.inner_puzzle_hash = TestRegistry::inner_puzzle_hash();

    let error = setup.verify(&response).await.unwrap_err();
    assert!(matches!(
        error,
        XchandlesError::ResolvedSingletonUnspentEve(..)
    ));
    assert!(!error.is_unavailable() && error.is_rejection());

    Ok(())
}

fn pending_transfer(setup: &Setup) -> PendingTransferResponse {
    PendingTransferResponse {
        new_owner_launcher_id: Bytes32::new([1; 32]),
        new_resolved_launcher_id: Bytes32::new([2; 32]),
        update_confirmation_height: 1,
        minimum_execution_height: 100,
        initiator_coin_id: setup.nft.coin.coin_id(),
        executor_coin_id: Bytes32::new([4; 32]),
    }
}

impl Setup {
    async fn verify_pending(
        &self,
        transfer: &PendingTransferResponse,
    ) -> Result<Option<Bytes32>, XchandlesError> {
        verify_pending_transfer(
            &self.peer,
            genesis_challenge(),
            self.registry.launcher_id,
            &self.handle,
            transfer,
        )
        .await
    }
}

#[tokio::test]
async fn test_pending_transfer_is_verified() -> anyhow::Result<()> {
    let mut setup = Setup::new().await?;
    let transfer = pending_transfer(&setup);

    // Nothing on chain records the transfer yet.
    assert_eq!(setup.verify_pending(&transfer).await?, None);

    let info = update_slot_info(setup.registry.launcher_id, &setup.handle, &transfer);
    let slot = setup.registry.create_update_slot(&setup.sim, info).await?;

    assert_eq!(
        setup.verify_pending(&transfer).await?,
        Some(slot.coin.coin_id())
    );

    // Any change to the claimed transfer gives a different slot, which doesn't exist.
    for tampered in [
        PendingTransferResponse {
            minimum_execution_height: 99,
            ..transfer
        },
        PendingTransferResponse {
            new_owner_launcher_id: Bytes32::new([9; 32]),
            ..transfer
        },
        PendingTransferResponse {
            new_resolved_launcher_id: Bytes32::new([9; 32]),
            ..transfer
        },
        PendingTransferResponse {
            initiator_coin_id: Bytes32::new([9; 32]),
            ..transfer
        },
    ] {
        assert_eq!(setup.verify_pending(&tampered).await?, None);
    }

    // The executor coin and confirmation height aren't part of the slot.
    let reindexed = PendingTransferResponse {
        update_confirmation_height: 7,
        executor_coin_id: Bytes32::new([8; 32]),
        ..transfer
    };
    assert_eq!(
        setup.verify_pending(&reindexed).await?,
        Some(slot.coin.coin_id())
    );

    Ok(())
}

#[tokio::test]
async fn test_forged_pending_transfer_is_not_verified() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let transfer = pending_transfer(&setup);

    // A coin with the update slot's puzzle hash, created by a standard coin rather than
    // the registry.
    let info = update_slot_info(setup.registry.launcher_id, &setup.handle, &transfer);
    let puzzle_hash = Slot::puzzle_hash(&info).into();

    let forger = setup.sim.lock().await.bls(1);
    let mut ctx = SpendContext::new();
    StandardLayer::new(forger.pk).spend(
        &mut ctx,
        forger.coin,
        Conditions::new().create_coin(puzzle_hash, 0, Memos::None),
    )?;
    setup
        .sim
        .lock()
        .await
        .spend_coins(ctx.take(), std::slice::from_ref(&forger.sk))?;

    assert_eq!(setup.verify_pending(&transfer).await?, None);

    Ok(())
}

/// Creates coins with the update slot's puzzle hash from a standard coin, as anyone can.
async fn forge_update_slots(
    setup: &Setup,
    transfer: &PendingTransferResponse,
    count: u64,
) -> anyhow::Result<()> {
    let info = update_slot_info(setup.registry.launcher_id, &setup.handle, transfer);
    let puzzle_hash: Bytes32 = Slot::puzzle_hash(&info).into();

    let forger = setup.sim.lock().await.bls((0..count).sum::<u64>().max(1));
    let mut ctx = SpendContext::new();
    let mut conditions = Conditions::new();
    for amount in 0..count {
        conditions = conditions.create_coin(puzzle_hash, amount, Memos::None);
    }
    StandardLayer::new(forger.pk).spend(&mut ctx, forger.coin, conditions)?;
    setup
        .sim
        .lock()
        .await
        .spend_coins(ctx.take(), std::slice::from_ref(&forger.sk))?;

    Ok(())
}

#[tokio::test]
async fn test_forged_update_slots_cannot_hide_a_pending_transfer() -> anyhow::Result<()> {
    let mut setup = Setup::new().await?;
    let mut transfer = pending_transfer(&setup);

    let info = update_slot_info(setup.registry.launcher_id, &setup.handle, &transfer);
    let slot = setup.registry.create_update_slot(&setup.sim, info).await?;
    forge_update_slots(&setup, &transfer, MAX_UPDATE_SLOT_CANDIDATES as u64 * 2).await?;

    // The real slot is checked first when the API reports the height it was created at.
    transfer.update_confirmation_height = setup
        .sim
        .lock()
        .await
        .coin_state(slot.coin.coin_id())
        .and_then(|state| state.created_height)
        .unwrap();

    assert_eq!(
        setup.verify_pending(&transfer).await?,
        Some(slot.coin.coin_id())
    );

    Ok(())
}

#[tokio::test]
async fn test_too_many_forged_update_slots_are_not_called_unverified() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let transfer = pending_transfer(&setup);

    // With only a few forgeries, all of them are checked and the claim is unverified.
    forge_update_slots(&setup, &transfer, 2).await?;
    assert_eq!(setup.verify_pending(&transfer).await?, None);

    // With more than can be checked, the transfer can't be ruled out.
    forge_update_slots(&setup, &transfer, MAX_UPDATE_SLOT_CANDIDATES as u64).await?;
    let error = setup.verify_pending(&transfer).await.unwrap_err();
    assert!(matches!(error, XchandlesError::TooManyUpdateSlotCandidates));
    assert!(error.is_unavailable());

    Ok(())
}

#[tokio::test]
async fn test_pending_transfer_for_another_registry_is_not_verified() -> anyhow::Result<()> {
    let setup = Setup::new().await?;
    let transfer = pending_transfer(&setup);

    // Another registry records the same transfer under its own launcher ID.
    let mut other = TestRegistry::launch(&setup.sim).await?;
    let info = update_slot_info(other.launcher_id, &setup.handle, &transfer);
    let slot = other.create_update_slot(&setup.sim, info).await?;
    assert!(
        setup
            .sim
            .lock()
            .await
            .coin_state(slot.coin.coin_id())
            .is_some()
    );

    assert_eq!(setup.verify_pending(&transfer).await?, None);

    Ok(())
}

#[test]
fn test_handle_slot_matches_live_registries() -> anyhow::Result<()> {
    use hex_literal::hex;
    use reqwest::StatusCode;

    for (registry_launcher_id, fixture, slot_coin_id) in [
        (
            hex!("6f314f69a9cf776311e1c7781a3013e4daae3696c71d8a04d049a5af1530f050"),
            include_str!("fixtures/handle_yakuhito.json"),
            hex!("7e034d4b635e5bc780da9a8c3f4bf84afeb3982bbcb03b604f2bfcca3b3c6c31"),
        ),
        (
            hex!("c98d2ea5fbaa6507c2d56598b81833c6c643f1ead0062c4641ac1f9d94331c4a"),
            include_str!("fixtures/testnet_handle_cloudwallet.json"),
            hex!("6df254042ea62986eb5f86bfac3b8dc2e9caf1fd002601c62df0a51281532153"),
        ),
    ] {
        let response: HandleResponse = decode(StatusCode::OK, fixture.as_bytes())?;
        let slot = handle_slot(registry_launcher_id.into(), &response)?;
        assert_eq!(slot.coin.coin_id(), Bytes32::new(slot_coin_id));

        // The same response can't be passed off as another registry's.
        assert!(matches!(
            handle_slot(Bytes32::new([1; 32]), &response),
            Err(XchandlesError::InvalidLineage)
        ));
    }

    Ok(())
}

mod store {
    use sage_database::{Database, XchandleRow, XchandleStatus};
    use sqlx::{migrate, sqlite::SqlitePoolOptions};

    use super::*;

    const NOW: u64 = 1_800_000_000;

    async fn test_db() -> anyhow::Result<Database> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;
        migrate!("../../migrations").run(&pool).await?;
        Ok(Database::new(pool))
    }

    fn registry() -> Bytes32 {
        Bytes32::new([0xaa; 32])
    }

    fn handle() -> Handle {
        Handle::parse("alice").unwrap()
    }

    fn verified(resolved_puzzle_hash: Bytes32) -> VerifiedHandle {
        let handle = handle();
        let resolved_launcher_id = Bytes32::new([resolved_puzzle_hash.to_bytes()[0]; 32]);

        VerifiedHandle {
            registry_launcher_id: registry(),
            slot_value: XchandlesHandleSlotValue::new(
                3,
                handle.hash(),
                Bytes32::new([1; 32]),
                Bytes32::new([2; 32]),
                NOW + 30 * 24 * 60 * 60,
                resolved_launcher_id,
                resolved_launcher_id,
            ),
            handle,
            slot_proof: LineageProof {
                parent_parent_coin_info: Bytes32::new([3; 32]),
                parent_inner_puzzle_hash: Bytes32::new([4; 32]),
                parent_amount: 1,
            },
            slot_parent_id: Bytes32::new([5; 32]),
            slot_coin_id: Bytes32::new([6; 32]),
            slot_confirmation_height: 100,
            resolved_kind: ResolvedSingletonKind::Nft,
            resolved_coin_id: Bytes32::new([7; 32]),
            resolved_confirmation_height: 90,
            resolved_puzzle_hash,
            indexed_height: 110,
            tip: ChainTip {
                height: 110,
                timestamp: NOW - 60,
            },
        }
    }

    async fn store(
        db: &Database,
        now: u64,
        result: Result<VerifiedHandle, XchandlesError>,
    ) -> Result<HandleResolution, XchandlesError> {
        resolve_and_store(db, registry(), &handle(), now, async { result }).await
    }

    fn unavailable() -> XchandlesError {
        XchandlesError::Api(XchandlesApiError::IndexStale)
    }

    const A: Bytes32 = Bytes32::new([0x0a; 32]);
    const B: Bytes32 = Bytes32::new([0x0b; 32]);
    const C: Bytes32 = Bytes32::new([0x0c; 32]);

    #[tokio::test]
    async fn test_stores_verified_mapping() -> anyhow::Result<()> {
        let db = test_db().await?;

        let resolution = store(&db, NOW, Ok(verified(A))).await?;
        assert!(!resolution.cached);
        assert_eq!(resolution.changed_from(), None);

        let row = db.xchandle(registry(), "alice").await?.unwrap();
        assert_eq!(row, resolution.row);
        assert_eq!(row.resolved_puzzle_hash, A);
        assert_eq!(row.status, XchandleStatus::Verified);
        assert_eq!(row.slot_counter, 3);
        assert_eq!(row.slot_parent_id, Bytes32::new([5; 32]));
        assert_eq!(row.verified_height, 110);
        assert_eq!(row.last_verified_at, NOW);

        assert_eq!(db.xchandles_by_puzzle_hash(registry(), A).await?, vec![row]);
        assert!(db.xchandles_by_puzzle_hash(registry(), B).await?.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_changed_mapping_warns_until_acknowledged() -> anyhow::Result<()> {
        let db = test_db().await?;

        store(&db, NOW, Ok(verified(A))).await?;
        db.set_xchandle_last_sent(registry(), "alice", A).await?;

        // Re-verifying an unchanged mapping, for example after a neighbor recreated the
        // slot, doesn't warn.
        let resolution = store(&db, NOW, Ok(verified(A))).await?;
        assert_eq!(resolution.changed_from(), None);

        let resolution = store(&db, NOW, Ok(verified(B))).await?;
        assert_eq!(resolution.row.resolved_puzzle_hash, B);
        assert_eq!(resolution.changed_from(), Some(A));
        assert_eq!(
            resolution.row.previous_resolved_launcher_id,
            Some(Bytes32::new([0x0a; 32]))
        );
        assert_eq!(resolution.row.last_sent_puzzle_hash, Some(A));

        // The warning persists, and keeps the address the user last knew about.
        let resolution = store(&db, NOW, Ok(verified(B))).await?;
        assert_eq!(resolution.changed_from(), Some(A));
        let resolution = store(&db, NOW, Ok(verified(C))).await?;
        assert_eq!(resolution.changed_from(), Some(A));

        db.acknowledge_xchandle_change(registry(), "alice").await?;
        let resolution = store(&db, NOW, Ok(verified(C))).await?;
        assert_eq!(resolution.changed_from(), None);

        // Changing back to the last known address clears the warning.
        let resolution = store(&db, NOW, Ok(verified(A))).await?;
        assert_eq!(resolution.changed_from(), Some(C));
        let resolution = store(&db, NOW, Ok(verified(C))).await?;
        assert_eq!(resolution.changed_from(), None);

        Ok(())
    }

    /// Addresses are labelled from `xchandles_by_puzzle_hash`, so a handle must stop labelling
    /// an address as soon as its resolved NFT is found to have moved, and only label the new
    /// one.
    #[tokio::test]
    async fn test_resolved_nft_transfer_moves_address_lookup() -> anyhow::Result<()> {
        let db = test_db().await?;

        let names = |rows: Vec<XchandleRow>| {
            rows.into_iter()
                .map(|row| row.handle)
                .collect::<Vec<String>>()
        };

        store(&db, NOW, Ok(verified(A))).await?;

        // A second handle resolving to the same address.
        let bob = Handle::parse("bob").unwrap();
        let mut bob_verified = verified(A);
        bob_verified.slot_value.handle_hash = bob.hash();
        bob_verified.handle = bob.clone();
        resolve_and_store(&db, registry(), &bob, NOW, async { Ok(bob_verified) }).await?;

        assert_eq!(
            names(db.xchandles_by_puzzle_hash(registry(), A).await?),
            ["alice", "bob"]
        );

        // Alice's resolved NFT is transferred, and the next lookup verifies the new owner.
        store(&db, NOW + 60, Ok(verified(B))).await?;

        assert_eq!(
            names(db.xchandles_by_puzzle_hash(registry(), A).await?),
            ["bob"]
        );
        let moved = db.xchandles_by_puzzle_hash(registry(), B).await?;
        assert_eq!(names(moved.clone()), ["alice"]);
        assert_eq!(moved[0].previous_resolved_puzzle_hash, Some(A));

        // Once a lookup is rejected, the mapping no longer labels any address.
        store(&db, NOW + 120, Err(XchandlesError::InvalidLineage))
            .await
            .unwrap_err();
        assert!(db.xchandles_by_puzzle_hash(registry(), B).await?.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_offline_fallback() -> anyhow::Result<()> {
        let db = test_db().await?;

        // Nothing to fall back to.
        assert!(store(&db, NOW, Err(unavailable())).await.is_err());

        let verified_row = store(&db, NOW, Ok(verified(A))).await?.row;

        let resolution = store(&db, NOW + 60, Err(unavailable())).await?;
        assert!(resolution.cached);
        assert_eq!(resolution.row, verified_row);

        let resolution = store(
            &db,
            NOW + OFFLINE_MAX_AGE_SECONDS,
            Err(XchandlesError::NoPeers),
        )
        .await?;
        assert!(resolution.cached);

        // Too old.
        let error = store(&db, NOW + OFFLINE_MAX_AGE_SECONDS + 1, Err(unavailable()))
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            XchandlesError::Api(XchandlesApiError::IndexStale)
        ));

        // Expired by the local clock, even though the mapping is recent.
        let mut expiring = verified(B);
        expiring.slot_value.expiration = NOW + 10;
        store(&db, NOW, Ok(expiring)).await?;
        assert!(store(&db, NOW + 5, Err(unavailable())).await.is_ok());
        assert!(store(&db, NOW + 10, Err(unavailable())).await.is_err());

        // The offline fallback never changes the stored mapping.
        let row = db.xchandle(registry(), "alice").await?.unwrap();
        assert_eq!(row.status, XchandleStatus::Verified);

        Ok(())
    }

    #[tokio::test]
    async fn test_rejected_lookup_marks_mapping_unverified() -> anyhow::Result<()> {
        let db = test_db().await?;

        store(&db, NOW, Ok(verified(A))).await?;

        let error = store(&db, NOW, Err(XchandlesError::InvalidLineage))
            .await
            .unwrap_err();
        assert!(matches!(error, XchandlesError::InvalidLineage));

        let row = db.xchandle(registry(), "alice").await?.unwrap();
        assert_eq!(row.status, XchandleStatus::Unverified);
        assert!(row.status_reason.is_some());
        // The last verified value is kept for comparison.
        assert_eq!(row.resolved_puzzle_hash, A);
        assert!(db.xchandles_by_puzzle_hash(registry(), A).await?.is_empty());

        // An unverified mapping can't be used offline.
        assert!(store(&db, NOW, Err(unavailable())).await.is_err());

        // A later verification still compares against the last verified address.
        let resolution = store(&db, NOW, Ok(verified(B))).await?;
        assert_eq!(resolution.row.status, XchandleStatus::Verified);
        assert_eq!(resolution.row.status_reason, None);
        assert_eq!(resolution.changed_from(), Some(A));

        // A handle which is no longer registered is also marked unverified.
        let error = store(
            &db,
            NOW,
            Err(XchandlesError::Api(XchandlesApiError::HandleExpired)),
        )
        .await
        .unwrap_err();
        assert!(error.is_not_registered());
        let row = db.xchandle(registry(), "alice").await?.unwrap();
        assert_eq!(row.status, XchandleStatus::Unverified);

        Ok(())
    }

    #[tokio::test]
    async fn test_expired_by_local_clock_is_rejected() -> anyhow::Result<()> {
        let db = test_db().await?;

        store(&db, NOW, Ok(verified(A))).await?;

        // The peer's peak is behind, so the verification didn't see the expiration, but the
        // local clock has already passed it.
        let expired = verified(A);
        let now = expired.expiration();
        let error = store(&db, now, Ok(expired)).await.unwrap_err();
        assert!(matches!(error, XchandlesError::Expired { timestamp, .. } if timestamp == now));

        let row = db.xchandle(registry(), "alice").await?.unwrap();
        assert_eq!(row.status, XchandleStatus::Unverified);
        assert_eq!(row.last_verified_at, NOW);

        Ok(())
    }

    #[tokio::test]
    async fn test_outdated_owner_is_not_used_offline() -> anyhow::Result<()> {
        let db = test_db().await?;

        for error in [
            XchandlesError::ResolvedSingletonMovedTooOften(Bytes32::new([1; 32])),
            XchandlesError::ResolvedSingletonUnspentEve(Bytes32::new([1; 32])),
        ] {
            store(&db, NOW, Ok(verified(A))).await?;

            // The current owner couldn't be verified, but the stored one is known to be out
            // of date, so it mustn't be returned when the API is unavailable later.
            store(&db, NOW, Err(error)).await.unwrap_err();
            let row = db.xchandle(registry(), "alice").await?.unwrap();
            assert_eq!(row.status, XchandleStatus::Unverified);
            assert!(store(&db, NOW, Err(unavailable())).await.is_err());
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_rejection_is_kept_if_status_cannot_be_stored() -> anyhow::Result<()> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;
        let db = Database::new(pool.clone());
        pool.close().await;

        let error = store(&db, NOW, Err(XchandlesError::InvalidLineage))
            .await
            .unwrap_err();
        assert!(matches!(error, XchandlesError::InvalidLineage));

        Ok(())
    }

    #[tokio::test]
    async fn test_unavailable_api_leaves_mapping_untouched() -> anyhow::Result<()> {
        let db = test_db().await?;

        let verified_row = store(&db, NOW, Ok(verified(A))).await?.row;

        for error in [
            XchandlesError::Api(XchandlesApiError::Status {
                status: 429,
                code: "rate_limited".to_string(),
                message: String::new(),
            }),
            XchandlesError::Api(XchandlesApiError::InvalidResponse(
                serde_json::from_str::<HandleResponse>("<html>").unwrap_err(),
            )),
            XchandlesError::Api(XchandlesApiError::InvalidUrl("not a url".to_string())),
            XchandlesError::ResolutionUnavailable,
            XchandlesError::SlotSpent(Bytes32::default()),
            XchandlesError::Timeout,
        ] {
            let resolution = store(&db, NOW, Err(error)).await?;
            assert!(resolution.cached);
        }

        let row = db.xchandle(registry(), "alice").await?.unwrap();
        assert_eq!(row, verified_row);

        // Errors which are neither leave it untouched too.
        let error = store(&db, NOW, Err(XchandlesError::UnsupportedNetwork))
            .await
            .unwrap_err();
        assert!(matches!(error, XchandlesError::UnsupportedNetwork));
        assert_eq!(
            db.xchandle(registry(), "alice").await?.unwrap(),
            verified_row
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_writes_during_lookup_are_kept() -> anyhow::Result<()> {
        let db = test_db().await?;

        store(&db, NOW, Ok(verified(A))).await?;
        store(&db, NOW, Ok(verified(B))).await?;

        // The address is sent to and the change acknowledged while a lookup is in flight.
        let resolution = resolve_and_store(&db, registry(), &handle(), NOW, async {
            db.set_xchandle_last_sent(registry(), "alice", B).await?;
            db.acknowledge_xchandle_change(registry(), "alice").await?;
            Ok(verified(B))
        })
        .await?;

        assert_eq!(resolution.row.last_sent_puzzle_hash, Some(B));
        assert_eq!(resolution.changed_from(), None);
        assert_eq!(
            db.xchandle(registry(), "alice").await?.unwrap(),
            resolution.row
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_reorg_invalidates_mapping() -> anyhow::Result<()> {
        let db = test_db().await?;

        store(&db, NOW, Ok(verified(A))).await?;

        // A reorg above everything the mapping was verified against changes nothing.
        db.invalidate_xchandles_between(110, 120).await?;
        let row = db.xchandle(registry(), "alice").await?.unwrap();
        assert_eq!(row.status, XchandleStatus::Verified);

        // Neither does a reorg on a peer which is behind everything it was verified against.
        db.invalidate_xchandles_between(80, 89).await?;
        let row = db.xchandle(registry(), "alice").await?.unwrap();
        assert_eq!(row.status, XchandleStatus::Verified);

        // A reorg of the block the slot was confirmed in does, even if the peer is behind
        // the peak it was verified at.
        db.invalidate_xchandles_between(99, 100).await?;
        let row = db.xchandle(registry(), "alice").await?.unwrap();
        assert_eq!(row.status, XchandleStatus::Stale);
        assert!(store(&db, NOW, Err(unavailable())).await.is_err());

        // Verifying again restores it.
        let resolution = store(&db, NOW, Ok(verified(A))).await?;
        assert_eq!(resolution.row.status, XchandleStatus::Verified);
        assert_eq!(resolution.changed_from(), None);

        Ok(())
    }

    #[tokio::test]
    async fn test_mappings_are_isolated_by_registry() -> anyhow::Result<()> {
        let db = test_db().await?;
        let other_registry = Bytes32::new([0xbb; 32]);

        store(&db, NOW, Ok(verified(A))).await?;

        assert!(db.xchandle(other_registry, "alice").await?.is_none());
        assert!(
            db.xchandles_by_puzzle_hash(other_registry, A)
                .await?
                .is_empty()
        );

        let error = resolve_and_store(&db, other_registry, &handle(), NOW, async {
            Err(unavailable())
        })
        .await
        .unwrap_err();
        assert!(error.is_unavailable());

        Ok(())
    }
}
