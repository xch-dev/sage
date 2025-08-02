use chia::protocol::{Bytes, Bytes32};
use chia_wallet_sdk::{
    driver::{ClawbackV2, SpendContext},
    prelude::Memos,
};

use crate::WalletError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hint {
    None,
    P2PuzzleHash(Bytes32),
    Clawback(ClawbackV2),
}

pub fn calculate_memos(
    ctx: &mut SpendContext,
    hint: Hint,
    memos: Vec<Bytes>,
) -> Result<Memos, WalletError> {
    let mut result = Vec::new();

    match hint {
        Hint::None => {}
        Hint::P2PuzzleHash(p2_puzzle_hash) => {
            result.push(ctx.alloc(&p2_puzzle_hash)?);
        }
        Hint::Clawback(clawback) => {
            result.push(ctx.alloc(&clawback.receiver_puzzle_hash)?);
            result.push(ctx.alloc(&clawback.memo())?);
        }
    }

    for memo in memos {
        result.push(ctx.alloc(&memo)?);
    }

    Ok(if result.is_empty() {
        Memos::None
    } else {
        ctx.memos(&result)?
    })
}
