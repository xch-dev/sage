use chia::protocol::Bytes32;

/// Represents an asset id for a fungible asset or a launcher id for a singleton.
///
/// The [`Existing`](Id::Existing) variant represents an asset that is already in the wallet.
/// The [`New`](Id::New) variant represents a new asset that is not yet in the wallet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Id {
    /// An existing asset id.
    Existing(Bytes32),
    /// A new asset id, which is the index of the action in the transaction that creates it.
    New(usize),
}
