mod cat;
mod cat_coin;
mod coin_state;
mod collection;
mod did;
mod did_coin;
mod nft;
mod nft_coin;
mod nft_data;
mod nft_uri;

pub use cat::*;
pub use cat_coin::*;
pub use coin_state::*;
pub use collection::*;
pub use did::*;
pub use did_coin::*;
pub use nft::*;
pub use nft_coin::*;
pub use nft_data::*;
pub use nft_uri::*;

use crate::DatabaseError;

pub(crate) trait IntoRow {
    type Row;

    fn into_row(self) -> Result<Self::Row, DatabaseError>;
}

pub(crate) fn into_row<T: IntoRow>(t: T) -> Result<T::Row, DatabaseError> {
    t.into_row()
}
