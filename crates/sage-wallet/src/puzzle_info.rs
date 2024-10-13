use chia::{
    clvm_traits::{FromClvm, ToClvm},
    protocol::{Bytes32, Coin, Program},
    puzzles::{nft::NftMetadata, LineageProof, Proof},
};
use chia_wallet_sdk::{run_puzzle, Cat, Condition, Did, DidInfo, HashedPtr, Nft, NftInfo, Puzzle};
use clvmr::Allocator;
use tracing::{debug_span, warn};

use crate::ParseError;

/// Information about the puzzle and lineage of a coin, to be inserted into the database alongside a coin.
#[allow(clippy::large_enum_variant)]
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
        data_hash: Option<Bytes32>,
        metadata_hash: Option<Bytes32>,
        license_hash: Option<Bytes32>,
        data_uris: Vec<String>,
        metadata_uris: Vec<String>,
        license_uris: Vec<String>,
    },
    /// The coin could not be parsed due to an error or it was a kind of puzzle we don't know about.
    Unknown { hint: Bytes32 },
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
            coin = %coin.coin_id()
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

        let output = run_puzzle(&mut allocator, parent_puzzle_ptr, parent_solution)
            .map_err(|_| ParseError::Eval)?;

        let conditions = Vec::<Condition>::from_clvm(&allocator, output)
            .map_err(|_| ParseError::InvalidConditions)?;

        let Some(mut create_coin) = conditions
            .into_iter()
            .filter_map(Condition::into_create_coin)
            .find(|cond| {
                cond.puzzle_hash == coin.puzzle_hash
                    && cond.amount == coin.amount
                    && !cond.memos.is_empty()
                    && cond.memos[0].len() == 32
            })
        else {
            return Err(ParseError::MissingHint);
        };

        let hint = Bytes32::try_from(create_coin.memos.remove(0).into_inner())
            .expect("the hint is always 32 bytes, as checked above");

        match Cat::parse_children(&mut allocator, parent_coin, parent_puzzle, parent_solution) {
            // If there was an error parsing the CAT, we can exit early.
            Err(error) => {
                warn!("Invalid CAT: {}", error);
                return Ok(Self::Unknown { hint });
            }

            // If the coin is a CAT coin, return the relevant information.
            Ok(Some(cats)) => {
                let Some(cat) = cats.into_iter().find(|cat| cat.coin == coin) else {
                    warn!("CAT coin not found in children");
                    return Ok(Self::Unknown { hint });
                };

                // We don't support parsing eve CATs during syncing.
                let Some(lineage_proof) = cat.lineage_proof else {
                    return Ok(Self::Unknown { hint });
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

        match Nft::<HashedPtr>::parse_child(
            &mut allocator,
            parent_coin,
            parent_puzzle,
            parent_solution,
        ) {
            // If there was an error parsing the NFT, we can exit early.
            Err(error) => {
                warn!("Invalid NFT: {}", error);
                return Ok(Self::Unknown { hint });
            }

            // If the coin is a NFT coin, return the relevant information.
            Ok(Some(nft)) => {
                if nft.coin != coin {
                    warn!("NFT coin does not match expected coin");
                    return Ok(Self::Unknown { hint });
                }

                // We don't support parsing eve NFTs during syncing.
                let Proof::Lineage(lineage_proof) = nft.proof else {
                    return Ok(Self::Unknown { hint });
                };

                let metadata = NftMetadata::from_clvm(&allocator, nft.info.metadata.ptr()).ok();

                let metadata_program = Program::from_clvm(&allocator, nft.info.metadata.ptr())
                    .map_err(|_| ParseError::Serialize)?;

                return Ok(Self::Nft {
                    lineage_proof,
                    info: nft.info.with_metadata(metadata_program),
                    data_hash: metadata.as_ref().and_then(|m| m.data_hash),
                    metadata_hash: metadata.as_ref().and_then(|m| m.metadata_hash),
                    license_hash: metadata.as_ref().and_then(|m| m.license_hash),
                    data_uris: metadata
                        .as_ref()
                        .map(|m| m.data_uris.clone())
                        .unwrap_or_default(),
                    metadata_uris: metadata
                        .as_ref()
                        .map(|m| m.metadata_uris.clone())
                        .unwrap_or_default(),
                    license_uris: metadata
                        .as_ref()
                        .map(|m| m.license_uris.clone())
                        .unwrap_or_default(),
                });
            }

            // If the coin is not a NFT coin, continue parsing.
            Ok(None) => {}
        }

        match Did::<HashedPtr>::parse_child(
            &mut allocator,
            parent_coin,
            parent_puzzle,
            parent_solution,
            coin,
        ) {
            // If there was an error parsing the DID, we can exit early.
            Err(error) => {
                warn!("Invalid DID: {}", error);
                return Ok(Self::Unknown { hint });
            }

            // If the coin is a DID coin, return the relevant information.
            Ok(Some(did)) => {
                // We don't support parsing eve DIDs during syncing.
                let Proof::Lineage(lineage_proof) = did.proof else {
                    return Ok(Self::Unknown { hint });
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

        Ok(Self::Unknown { hint })
    }
}
