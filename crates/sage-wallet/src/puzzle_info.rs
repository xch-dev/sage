use chia::{
    clvm_traits::{FromClvm, ToClvm},
    protocol::{Bytes32, Coin, Program},
    puzzles::{LineageProof, Proof},
};
use chia_wallet_sdk::{Cat, Did, DidInfo, HashedPtr, Nft, NftInfo, Primitive, Puzzle};
use clvmr::Allocator;
use tracing::{debug_span, warn};

use crate::ParseError;

/// Information about the puzzle and lineage of a coin, to be inserted into the database alongside a coin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PuzzleInfo {
    /// A non-eve coin which follows the CAT2 standard.
    Cat {
        asset_id: Bytes32,
        lineage_proof: LineageProof,
        p2_puzzle_hash: Bytes32,
    },
    /// A non-eve coin which follows the DID1 standard.
    Did {
        lineage_proof: LineageProof,
        info: DidInfo<Program>,
    },
    /// A non-eve coin which follows the NFT1 standard.
    Nft {
        lineage_proof: LineageProof,
        info: NftInfo<Program>,
    },
    /// The coin could not be parsed due to an error or it was a kind of puzzle we don't know about.
    Unknown,
}

impl PuzzleInfo {
    pub fn parse(
        parent_coin: Coin,
        parent_puzzle: &Program,
        parent_solution: &Program,
        coin: Coin,
    ) -> Result<Self, ParseError> {
        let parse_span = debug_span!(
            "parse",
            parent_coin = %parent_coin.coin_id(),
            coin_id = %coin.coin_id()
        );
        let _span = parse_span.enter();

        let mut allocator = Allocator::new();

        let parent_puzzle_ptr = parent_puzzle
            .to_clvm(&mut allocator)
            .map_err(|_| ParseError::AllocatePuzzle)?;

        let parent_puzzle = Puzzle::parse(&allocator, parent_puzzle_ptr);

        let parent_solution = parent_solution
            .to_clvm(&mut allocator)
            .map_err(|_| ParseError::AllocateSolution)?;

        match Cat::from_parent_spend(
            &mut allocator,
            parent_coin,
            parent_puzzle,
            parent_solution,
            coin,
        ) {
            // If there was an error parsing the CAT, we can exit early.
            Err(error) => {
                warn!("Invalid CAT: {}", error);
                return Ok(Self::Unknown);
            }

            // If the coin is a CAT coin, return the relevant information.
            Ok(Some(cat)) => {
                // We don't support parsing eve CATs during syncing.
                let Some(lineage_proof) = cat.lineage_proof else {
                    return Ok(Self::Unknown);
                };

                return Ok(Self::Cat {
                    asset_id: cat.asset_id,
                    lineage_proof,
                    p2_puzzle_hash: cat.p2_puzzle_hash,
                });
            }

            // If the coin is not a CAT coin, continue parsing.
            Ok(None) => {}
        }

        match Nft::<HashedPtr>::from_parent_spend(
            &mut allocator,
            parent_coin,
            parent_puzzle,
            parent_solution,
            coin,
        ) {
            // If there was an error parsing the NFT, we can exit early.
            Err(error) => {
                warn!("Invalid NFT: {}", error);
                return Ok(Self::Unknown);
            }

            // If the coin is a NFT coin, return the relevant information.
            Ok(Some(nft)) => {
                // We don't support parsing eve NFTs during syncing.
                let Proof::Lineage(lineage_proof) = nft.proof else {
                    return Ok(Self::Unknown);
                };

                let metadata = Program::from_clvm(&allocator, nft.info.metadata.ptr())
                    .map_err(|_| ParseError::Serialize)?;

                return Ok(Self::Nft {
                    lineage_proof,
                    info: nft.info.with_metadata(metadata),
                });
            }

            // If the coin is not a NFT coin, continue parsing.
            Ok(None) => {}
        }

        match Did::<HashedPtr>::from_parent_spend(
            &mut allocator,
            parent_coin,
            parent_puzzle,
            parent_solution,
            coin,
        ) {
            // If there was an error parsing the DID, we can exit early.
            Err(error) => {
                warn!("Invalid DID: {}", error);
                return Ok(Self::Unknown);
            }

            // If the coin is a DID coin, return the relevant information.
            Ok(Some(did)) => {
                // We don't support parsing eve DIDs during syncing.
                let Proof::Lineage(lineage_proof) = did.proof else {
                    return Ok(Self::Unknown);
                };

                let metadata = Program::from_clvm(&allocator, did.info.metadata.ptr())
                    .map_err(|_| ParseError::Serialize)?;

                return Ok(Self::Did {
                    lineage_proof,
                    info: did.info.with_metadata(metadata),
                });
            }

            // If the coin is not a DID coin, continue parsing.
            Ok(None) => {}
        }

        Ok(Self::Unknown)
    }
}
