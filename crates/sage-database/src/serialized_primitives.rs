use chia::{
    clvm_traits::{FromClvm, ToClvm},
    protocol::{Bytes32, Program},
};
use chia_wallet_sdk::{
    driver::{Did, DidInfo, DriverError, HashedPtr, Nft, NftInfo, Singleton},
    prelude::Allocator,
};

pub type SerializedNft = Singleton<SerializedNftInfo>;
pub type SerializedDid = Singleton<SerializedDidInfo>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerializedNftInfo {
    pub launcher_id: Bytes32,
    pub metadata: Program,
    pub metadata_updater_puzzle_hash: Bytes32,
    pub current_owner: Option<Bytes32>,
    pub royalty_puzzle_hash: Bytes32,
    pub royalty_basis_points: u16,
    pub p2_puzzle_hash: Bytes32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerializedDidInfo {
    pub launcher_id: Bytes32,
    pub recovery_list_hash: Option<Bytes32>,
    pub num_verifications_required: u64,
    pub metadata: Program,
    pub p2_puzzle_hash: Bytes32,
}

pub trait DeserializePrimitive {
    type Primitive;

    fn deserialize(self, allocator: &mut Allocator) -> Result<Self::Primitive, DriverError>;
}

impl DeserializePrimitive for SerializedNft {
    type Primitive = Nft;

    fn deserialize(self, allocator: &mut Allocator) -> Result<Self::Primitive, DriverError> {
        Ok(Nft::new(
            self.coin,
            self.proof,
            self.info.deserialize(allocator)?,
        ))
    }
}

impl DeserializePrimitive for SerializedNftInfo {
    type Primitive = NftInfo;

    fn deserialize(self, allocator: &mut Allocator) -> Result<Self::Primitive, DriverError> {
        let ptr = self.metadata.to_clvm(allocator)?;

        Ok(NftInfo::new(
            self.launcher_id,
            HashedPtr::from_ptr(allocator, ptr),
            self.metadata_updater_puzzle_hash,
            self.current_owner,
            self.royalty_puzzle_hash,
            self.royalty_basis_points,
            self.p2_puzzle_hash,
        ))
    }
}

impl DeserializePrimitive for SerializedDid {
    type Primitive = Did;

    fn deserialize(self, allocator: &mut Allocator) -> Result<Self::Primitive, DriverError> {
        Ok(Did::new(
            self.coin,
            self.proof,
            self.info.deserialize(allocator)?,
        ))
    }
}

impl DeserializePrimitive for SerializedDidInfo {
    type Primitive = DidInfo;

    fn deserialize(self, allocator: &mut Allocator) -> Result<Self::Primitive, DriverError> {
        let ptr = self.metadata.to_clvm(allocator)?;

        Ok(DidInfo::new(
            self.launcher_id,
            self.recovery_list_hash,
            self.num_verifications_required,
            HashedPtr::from_ptr(allocator, ptr),
            self.p2_puzzle_hash,
        ))
    }
}

pub trait SerializePrimitive: Sized {
    type Primitive;

    fn serialize(&self, allocator: &Allocator) -> Result<Self::Primitive, DriverError>;
}

impl SerializePrimitive for Nft {
    type Primitive = SerializedNft;

    fn serialize(&self, allocator: &Allocator) -> Result<Self::Primitive, DriverError> {
        Ok(SerializedNft {
            coin: self.coin,
            proof: self.proof,
            info: self.info.serialize(allocator)?,
        })
    }
}

impl SerializePrimitive for NftInfo {
    type Primitive = SerializedNftInfo;

    fn serialize(&self, allocator: &Allocator) -> Result<Self::Primitive, DriverError> {
        Ok(SerializedNftInfo {
            launcher_id: self.launcher_id,
            metadata: Program::from_clvm(allocator, self.metadata.ptr())?,
            metadata_updater_puzzle_hash: self.metadata_updater_puzzle_hash,
            current_owner: self.current_owner,
            royalty_puzzle_hash: self.royalty_puzzle_hash,
            royalty_basis_points: self.royalty_basis_points,
            p2_puzzle_hash: self.p2_puzzle_hash,
        })
    }
}

impl SerializePrimitive for Did {
    type Primitive = SerializedDid;

    fn serialize(&self, allocator: &Allocator) -> Result<Self::Primitive, DriverError> {
        Ok(SerializedDid {
            coin: self.coin,
            proof: self.proof,
            info: self.info.serialize(allocator)?,
        })
    }
}

impl SerializePrimitive for DidInfo {
    type Primitive = SerializedDidInfo;

    fn serialize(&self, allocator: &Allocator) -> Result<Self::Primitive, DriverError> {
        Ok(SerializedDidInfo {
            launcher_id: self.launcher_id,
            recovery_list_hash: self.recovery_list_hash,
            num_verifications_required: self.num_verifications_required,
            metadata: Program::from_clvm(allocator, self.metadata.ptr())?,
            p2_puzzle_hash: self.p2_puzzle_hash,
        })
    }
}
