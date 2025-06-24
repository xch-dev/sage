use chia::{
    clvm_traits::{FromClvm, ToClvm},
    protocol::Program,
    puzzles::nft::NftMetadata,
};
use chia_puzzles::SINGLETON_LAUNCHER_HASH;
use chia_wallet_sdk::driver::{CatInfo, DidInfo, HashedPtr, NftInfo, Puzzle};
use clvmr::Allocator;
use tracing::{debug_span, warn};

use crate::WalletError;

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum CoinKind {
    Unknown,
    Launcher,
    Cat {
        info: CatInfo,
    },
    Did {
        info: DidInfo<Program>,
    },
    Nft {
        info: NftInfo<Program>,
        metadata: Option<NftMetadata>,
    },
}

impl CoinKind {
    pub fn from_puzzle(puzzle: &Program) -> Result<Self, WalletError> {
        let mut allocator = Allocator::new();

        let puzzle_ptr = puzzle.to_clvm(&mut allocator)?;
        let puzzle = Puzzle::parse(&allocator, puzzle_ptr);

        Self::from_puzzle_cached(&allocator, puzzle)
    }

    pub fn from_puzzle_cached(allocator: &Allocator, puzzle: Puzzle) -> Result<Self, WalletError> {
        let parse_span = debug_span!("parse puzzle");
        let _span = parse_span.enter();

        if puzzle.curried_puzzle_hash() == SINGLETON_LAUNCHER_HASH.into() {
            return Ok(Self::Launcher);
        }

        match CatInfo::parse(allocator, puzzle) {
            // If there was an error parsing the CAT, we can exit early.
            Err(error) => {
                warn!("Invalid CAT: {}", error);
                return Ok(Self::Unknown);
            }

            // If the coin is a CAT coin, return the relevant information.
            Ok(Some((cat, _inner_puzzle))) => {
                return Ok(Self::Cat { info: cat });
            }

            // If the coin is not a CAT coin, continue parsing.
            Ok(None) => {}
        }

        match NftInfo::<HashedPtr>::parse(allocator, puzzle) {
            // If there was an error parsing the NFT, we can exit early.
            Err(error) => {
                warn!("Invalid NFT: {}", error);
                return Ok(Self::Unknown);
            }

            // If the coin is a NFT coin, return the relevant information.
            Ok(Some((nft, _inner_puzzle))) => {
                let metadata_program = Program::from_clvm(allocator, nft.metadata.ptr())?;
                let metadata = NftMetadata::from_clvm(allocator, nft.metadata.ptr()).ok();

                return Ok(Self::Nft {
                    info: nft.with_metadata(metadata_program),
                    metadata,
                });
            }

            // If the coin is not a NFT coin, continue parsing.
            Ok(None) => {}
        }

        match DidInfo::<HashedPtr>::parse(allocator, puzzle) {
            // If there was an error parsing the DID, we can exit early.
            Err(error) => {
                warn!("Invalid DID: {}", error);
                return Ok(Self::Unknown);
            }

            // If the coin is a DID coin, return the relevant information.
            Ok(Some((did, _inner_puzzle))) => {
                let metadata = Program::from_clvm(allocator, did.metadata.ptr())?;

                return Ok(Self::Did {
                    info: did.with_metadata(metadata),
                });
            }

            // If the coin is not a DID coin, continue parsing.
            Ok(None) => {}
        }

        Ok(Self::Unknown)
    }
}
