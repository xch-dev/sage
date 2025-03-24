mod cat;
mod cat_coin;
mod coin_state;
mod collection;
mod derivation;
mod did;
mod did_coin;
mod nft;
mod nft_coin;
mod offer;

pub use cat::*;
pub use cat_coin::*;
pub use coin_state::*;
pub use collection::*;
pub use derivation::*;
pub use did::*;
pub use did_coin::*;
pub use nft::*;
pub(crate) use nft_coin::*;
pub use offer::*;

use crate::DatabaseError;

pub(crate) trait IntoRow {
    type Row;

    fn into_row(self) -> Result<Self::Row, DatabaseError>;
}

pub(crate) fn into_row<T: IntoRow>(t: T) -> Result<T::Row, DatabaseError> {
    t.into_row()
}
