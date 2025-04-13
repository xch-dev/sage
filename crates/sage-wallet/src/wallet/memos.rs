use chia::protocol::{Bytes, Bytes32};
use chia_wallet_sdk::{driver::SpendContext, prelude::Memos};

use crate::WalletError;

pub fn calculate_memos(
    ctx: &mut SpendContext,
    p2_puzzle_hash: Bytes32,
    include_hint: bool,
    memos: Option<Vec<Bytes>>,
) -> Result<Option<Memos>, WalletError> {
    let mut result = None;

    if include_hint {
        result = Some(vec![p2_puzzle_hash.into()]);
    }

    if let Some(memos) = memos {
        if let Some(result) = result.as_mut() {
            result.extend(memos);
        } else {
            result = Some(memos);
        }
    }

    Ok(if let Some(result) = result {
        Some(ctx.memos(&result)?)
    } else {
        None
    })
}
