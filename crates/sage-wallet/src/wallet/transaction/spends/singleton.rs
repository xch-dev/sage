use std::mem;

use chia::{
    protocol::{Bytes32, Coin},
    puzzles::{singleton::SingletonArgs, Proof},
};
use chia_wallet_sdk::{
    driver::{
        did_puzzle_assertion, Did, DidInfo, DidOwner, HashedPtr, Launcher, Nft, NftInfo,
        OptionContract, OptionInfo, Spend, SpendContext, StandardLayer,
    },
    prelude::TransferNft,
    types::Conditions,
};

use crate::WalletError;

#[derive(Debug, Clone)]
pub struct SingletonLineage<T>
where
    T: SingletonCoinExt,
{
    items: Vec<SingletonItem<T>>,
    child_info: T::Info,
    was_created: bool,
}

#[derive(Debug, Clone)]
pub struct SingletonItem<T> {
    coin: T,
    p2: StandardLayer,
    conditions: Conditions,
    launcher_index: u64,
    needs_spend: bool,
}

pub trait SingletonCoinExt {
    type Info;

    fn coin_id(&self) -> Bytes32;
    #[must_use]
    fn child_with_info(&self, info: Self::Info) -> Self;
    fn info(&self) -> Self::Info;
    fn spend(
        &self,
        ctx: &mut SpendContext,
        p2: StandardLayer,
        conditions: Conditions,
    ) -> Result<(), WalletError>;
}

impl SingletonCoinExt for Nft<HashedPtr> {
    type Info = NftInfo<HashedPtr>;

    fn coin_id(&self) -> Bytes32 {
        self.coin.coin_id()
    }

    fn child_with_info(&self, info: Self::Info) -> Self {
        Nft {
            coin: Coin::new(
                self.coin.coin_id(),
                SingletonArgs::curry_tree_hash(info.launcher_id, info.inner_puzzle_hash()).into(),
                self.coin.amount,
            ),
            proof: Proof::Lineage(self.child_lineage_proof()),
            info,
        }
    }

    fn info(&self) -> Self::Info {
        self.info
    }

    fn spend(
        &self,
        ctx: &mut SpendContext,
        p2: StandardLayer,
        conditions: Conditions,
    ) -> Result<(), WalletError> {
        self.spend_with(ctx, &p2, conditions)?;
        Ok(())
    }
}

impl SingletonCoinExt for Did<HashedPtr> {
    type Info = DidInfo<HashedPtr>;

    fn coin_id(&self) -> Bytes32 {
        self.coin.coin_id()
    }

    fn child_with_info(&self, info: Self::Info) -> Self {
        Did {
            coin: Coin::new(
                self.coin.coin_id(),
                SingletonArgs::curry_tree_hash(info.launcher_id, info.inner_puzzle_hash()).into(),
                self.coin.amount,
            ),
            proof: Proof::Lineage(self.child_lineage_proof()),
            info,
        }
    }

    fn info(&self) -> Self::Info {
        self.info
    }

    fn spend(
        &self,
        ctx: &mut SpendContext,
        p2: StandardLayer,
        conditions: Conditions,
    ) -> Result<(), WalletError> {
        self.spend_with(ctx, &p2, conditions)?;
        Ok(())
    }
}

impl SingletonCoinExt for OptionContract {
    type Info = OptionInfo;

    fn coin_id(&self) -> Bytes32 {
        self.coin.coin_id()
    }

    fn child_with_info(&self, info: Self::Info) -> Self {
        OptionContract {
            coin: Coin::new(
                self.coin.coin_id(),
                SingletonArgs::curry_tree_hash(info.launcher_id, info.inner_puzzle_hash()).into(),
                self.coin.amount,
            ),
            proof: Proof::Lineage(self.child_lineage_proof()),
            info,
        }
    }

    fn info(&self) -> Self::Info {
        self.info
    }

    fn spend(
        &self,
        ctx: &mut SpendContext,
        p2: StandardLayer,
        conditions: Conditions,
    ) -> Result<(), WalletError> {
        self.spend_with(ctx, &p2, conditions)?;
        Ok(())
    }
}

impl<T> SingletonLineage<T>
where
    T: SingletonCoinExt + Copy,
{
    pub fn new(coin: T, p2: StandardLayer, was_created: bool, needs_spend: bool) -> Self {
        Self {
            items: vec![SingletonItem::new(coin, p2, needs_spend)],
            child_info: coin.info(),
            was_created,
        }
    }

    pub fn was_created(&self) -> bool {
        self.was_created
    }

    pub fn coin(&self) -> T {
        self.items.last().expect("no lineage").coin
    }

    pub fn current(&self) -> &SingletonItem<T> {
        self.items.last().expect("no lineage")
    }

    pub fn current_mut(&mut self) -> &mut SingletonItem<T> {
        self.items.last_mut().expect("no lineage")
    }

    pub fn iter(&self) -> impl Iterator<Item = &SingletonItem<T>> {
        self.items.iter()
    }
}

impl SingletonLineage<Did<HashedPtr>> {
    pub fn recreate(&mut self, ctx: &mut SpendContext) -> Result<(), WalletError> {
        self.recreate_impl(ctx, false)
    }

    fn recreate_impl(
        &mut self,
        ctx: &mut SpendContext,
        is_update_spend: bool,
    ) -> Result<(), WalletError> {
        if self.recovery_list_hash_changed() && !is_update_spend {
            let p2_puzzle_hash = self.current().coin.info.p2_puzzle_hash;
            let p2_puzzle_hash = mem::replace(&mut self.child_info.p2_puzzle_hash, p2_puzzle_hash);
            self.recreate_impl(ctx, true)?;
            self.child_info.p2_puzzle_hash = p2_puzzle_hash;
        }

        let child_info = self.child_info;
        let current = self.current_mut();
        let hint = ctx.hint(child_info.p2_puzzle_hash)?;

        current.conditions = mem::take(&mut current.conditions).create_coin(
            child_info.inner_puzzle_hash().into(),
            current.coin.coin.amount,
            Some(hint),
        );

        let child = SingletonItem::new(current.coin.child_with_info(child_info), current.p2, false);
        self.items.push(child);

        Ok(())
    }

    pub fn has_conditions(&self) -> bool {
        self.current().has_conditions()
    }

    pub fn set_recovery_list_hash(&mut self, recovery_list_hash: Option<Bytes32>) {
        self.child_info.recovery_list_hash = recovery_list_hash;
        self.current_mut().needs_spend = true;
    }

    pub fn recovery_list_hash_changed(&self) -> bool {
        let current = self.current();

        current.coin.info.recovery_list_hash != self.child_info.recovery_list_hash
    }

    pub fn set_p2_puzzle_hash(&mut self, p2_puzzle_hash: Bytes32) {
        self.child_info.p2_puzzle_hash = p2_puzzle_hash;
        self.current_mut().needs_spend = true;
    }

    pub fn authorize_nft_ownership(&mut self, nft_puzzle_hash: Bytes32, nft_launcher_id: Bytes32) {
        let current = self.current_mut();

        current.conditions = mem::take(&mut current.conditions)
            .assert_puzzle_announcement(did_puzzle_assertion(
                nft_puzzle_hash,
                &TransferNft::new(
                    Some(current.coin.info.launcher_id),
                    Vec::new(),
                    Some(current.coin.info.inner_puzzle_hash().into()),
                ),
            ))
            .create_puzzle_announcement(nft_launcher_id.into());

        current.needs_spend = true;
    }

    pub fn add_conditions(&mut self, conditions: Conditions) {
        let current = self.current_mut();
        current.conditions = mem::take(&mut current.conditions).extend(conditions);
        current.needs_spend = true;
    }
}

impl SingletonLineage<Nft<HashedPtr>> {
    pub fn recreate(&mut self, ctx: &mut SpendContext) -> Result<(), WalletError> {
        let child_info = self.child_info;
        let current = self.current_mut();
        let hint = ctx.hint(child_info.p2_puzzle_hash)?;

        current.conditions = mem::take(&mut current.conditions).create_coin(
            child_info.p2_puzzle_hash,
            current.coin.coin.amount,
            Some(hint),
        );

        let child = SingletonItem::new(current.coin.child_with_info(child_info), current.p2, false);
        self.items.push(child);

        Ok(())
    }

    pub fn has_conditions(&self) -> bool {
        self.current().has_conditions()
    }

    pub fn set_p2_puzzle_hash(&mut self, p2_puzzle_hash: Bytes32) {
        self.child_info.p2_puzzle_hash = p2_puzzle_hash;
        self.current_mut().needs_spend = true;
    }

    pub fn set_did_owner(
        &mut self,
        ctx: &mut SpendContext,
        owner: Option<DidOwner>,
    ) -> Result<(), WalletError> {
        if self.did_owner_changed() {
            self.recreate(ctx)?;
        }

        self.child_info.current_owner = owner.map(|owner| owner.did_id);

        let current = self.current_mut();

        current.conditions = mem::take(&mut current.conditions).transfer_nft(
            owner.map(|owner| owner.did_id),
            Vec::new(),
            owner.map(|owner| owner.inner_puzzle_hash),
        );

        current.needs_spend = true;

        Ok(())
    }

    pub fn did_owner_changed(&self) -> bool {
        let current = self.current();

        current.coin.info.current_owner != self.child_info.current_owner
    }

    pub fn set_metadata(
        &mut self,
        ctx: &mut SpendContext,
        metadata_update: Spend,
        metadata: HashedPtr,
        metadata_updater_puzzle_hash: Bytes32,
    ) -> Result<(), WalletError> {
        if self.metadata_changed() {
            self.recreate(ctx)?;
        }

        self.child_info.metadata = metadata;
        self.child_info.metadata_updater_puzzle_hash = metadata_updater_puzzle_hash;

        let current = self.current_mut();

        current.conditions = mem::take(&mut current.conditions)
            .update_nft_metadata(metadata_update.puzzle, metadata_update.solution);

        current.needs_spend = true;

        Ok(())
    }

    pub fn metadata_changed(&self) -> bool {
        let current = self.current();

        current.coin.info.metadata != self.child_info.metadata
            || current.coin.info.metadata_updater_puzzle_hash
                != self.child_info.metadata_updater_puzzle_hash
    }
}

impl SingletonLineage<OptionContract> {
    pub fn recreate(&mut self, ctx: &mut SpendContext) -> Result<(), WalletError> {
        let child_info = self.child_info;
        let current = self.current_mut();
        let hint = ctx.hint(child_info.p2_puzzle_hash)?;

        current.conditions = mem::take(&mut current.conditions).create_coin(
            child_info.p2_puzzle_hash,
            current.coin.coin.amount,
            Some(hint),
        );

        let child = SingletonItem::new(current.coin.child_with_info(child_info), current.p2, false);
        self.items.push(child);

        Ok(())
    }

    pub fn has_conditions(&self) -> bool {
        self.current().has_conditions()
    }

    pub fn set_p2_puzzle_hash(&mut self, p2_puzzle_hash: Bytes32) {
        self.child_info.p2_puzzle_hash = p2_puzzle_hash;
        self.current_mut().needs_spend = true;
    }
}

impl<T> SingletonItem<T>
where
    T: SingletonCoinExt,
{
    pub fn new(coin: T, p2: StandardLayer, needs_spend: bool) -> Self {
        Self {
            coin,
            p2,
            conditions: Conditions::new(),
            launcher_index: 0,
            needs_spend,
        }
    }

    pub fn needs_spend(&self) -> bool {
        self.needs_spend
    }

    pub fn spend(
        &self,
        ctx: &mut SpendContext,
        extra_conditions: Conditions,
    ) -> Result<(), WalletError> {
        self.coin.spend(
            ctx,
            self.p2,
            self.conditions.clone().extend(extra_conditions),
        )
    }

    pub fn coin_id(&self) -> Bytes32 {
        self.coin.coin_id()
    }

    pub fn p2(&self) -> StandardLayer {
        self.p2
    }

    pub fn has_conditions(&self) -> bool {
        !self.conditions.is_empty()
    }

    pub fn create_launcher(&mut self) -> Launcher {
        let launcher_amount = self.launcher_index * 2;
        self.launcher_index += 1;

        let (create_launcher, launcher) =
            Launcher::create_early(self.coin.coin_id(), launcher_amount);

        self.conditions = mem::take(&mut self.conditions).extend(create_launcher);

        launcher.with_singleton_amount(1)
    }
}
