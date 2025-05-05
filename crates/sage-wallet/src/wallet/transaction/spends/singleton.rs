use std::mem;

use chia::{
    protocol::{Bytes32, Coin},
    puzzles::{singleton::SingletonArgs, Proof},
};
use chia_wallet_sdk::{
    driver::{
        Did, DidInfo, HashedPtr, Launcher, Nft, NftInfo, OptionContract, OptionInfo, StandardLayer,
    },
    types::Conditions,
};

#[derive(Debug, Clone)]
pub struct Singleton<T>
where
    T: SingletonCoinExt,
{
    pub coin: T,
    pub child_info: T::Info,
    pub p2: StandardLayer,
    pub conditions: Conditions,
    pub launcher_index: u64,
    pub was_created: bool,
}

pub trait SingletonCoinExt {
    type Info;

    fn coin_id(&self) -> Bytes32;
    #[must_use]
    fn child_with_info(&self, info: Self::Info) -> Self;
    fn info(&self) -> Self::Info;
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
}

impl<T> Singleton<T>
where
    T: SingletonCoinExt + Copy,
    T::Info: Copy,
{
    pub fn new(coin: T, child_info: T::Info, p2: StandardLayer, was_created: bool) -> Self {
        Self {
            coin,
            child_info,
            p2,
            conditions: Conditions::new(),
            launcher_index: 0,
            was_created,
        }
    }

    #[must_use]
    pub fn child_with(&self, coin: T) -> Self {
        Self::new(coin, coin.info(), self.p2, self.was_created)
    }

    #[must_use]
    pub fn child(&self) -> Self {
        self.child_with(self.coin.child_with_info(self.child_info))
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
