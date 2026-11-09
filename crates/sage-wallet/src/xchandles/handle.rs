use std::{fmt, str::FromStr};

use chia_wallet_sdk::{clvm_utils::tree_hash_atom, prelude::*};
use thiserror::Error;

pub const MIN_HANDLE_LENGTH: usize = 3;
pub const MAX_HANDLE_LENGTH: usize = 63;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum HandleError {
    #[error("Handles must be between {MIN_HANDLE_LENGTH} and {MAX_HANDLE_LENGTH} characters long")]
    InvalidLength,

    #[error("Handles may only contain lowercase letters and digits")]
    InvalidCharacter,
}

/// A syntactically valid XCHandles handle, stored without the leading `@`.
///
/// Handles are not normalized: `@Alice` is rejected rather than lowercased, since a
/// lookalike of a different handle must never resolve to it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Handle(String);

impl Handle {
    /// Parses a handle, accepting an optional leading `@`.
    pub fn parse(input: &str) -> Result<Self, HandleError> {
        let handle = input.strip_prefix('@').unwrap_or(input);

        if !(MIN_HANDLE_LENGTH..=MAX_HANDLE_LENGTH).contains(&handle.len()) {
            return Err(HandleError::InvalidLength);
        }

        if !handle
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        {
            return Err(HandleError::InvalidCharacter);
        }

        Ok(Self(handle.to_string()))
    }

    /// Parses user input as a handle only if it is written as `@handle`, so that other
    /// recipient formats such as addresses are never mistaken for handles.
    pub fn from_input(input: &str) -> Option<Result<Self, HandleError>> {
        input.starts_with('@').then(|| Self::parse(input))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The hash the registry stores the handle under, which is the tree hash of the
    /// handle as an atom.
    pub fn hash(&self) -> Bytes32 {
        tree_hash_atom(self.0.as_bytes()).into()
    }
}

impl fmt::Display for Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@{}", self.0)
    }
}

impl FromStr for Handle {
    type Err = HandleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use chia_wallet_sdk::chia::sha2::Sha256;
    use hex_literal::hex;

    use super::*;

    #[test]
    fn test_parse_valid_handles() {
        for input in ["abc", "@abc", "yakuhito", "123", "a1b2c3", &"a".repeat(63)] {
            let handle = Handle::parse(input).unwrap();
            assert_eq!(handle.as_str(), input.trim_start_matches('@'));
        }
    }

    #[test]
    fn test_parse_invalid_handles() {
        for (input, error) in [
            ("", HandleError::InvalidLength),
            ("@", HandleError::InvalidLength),
            ("ab", HandleError::InvalidLength),
            ("@ab", HandleError::InvalidLength),
            (&"a".repeat(64), HandleError::InvalidLength),
            ("Alice", HandleError::InvalidCharacter),
            ("ali-ce", HandleError::InvalidCharacter),
            ("ali_ce", HandleError::InvalidCharacter),
            ("ali.xch", HandleError::InvalidCharacter),
            (" alice", HandleError::InvalidCharacter),
            ("alice ", HandleError::InvalidCharacter),
            ("@@alice", HandleError::InvalidCharacter),
            ("ālice", HandleError::InvalidCharacter),
        ] {
            assert_eq!(Handle::parse(input), Err(error), "{input:?}");
        }
    }

    #[test]
    fn test_from_input_requires_at_sign() {
        assert_eq!(Handle::from_input("alice"), None);
        assert_eq!(
            Handle::from_input("xch1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqmlvj3x"),
            None
        );
        assert_eq!(
            Handle::from_input("@alice"),
            Some(Ok(Handle("alice".to_string())))
        );
        assert_eq!(
            Handle::from_input("@Al"),
            Some(Err(HandleError::InvalidLength))
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(Handle::parse("alice").unwrap().to_string(), "@alice");
    }

    #[test]
    fn test_handle_hash() {
        let handle = Handle::parse("yakuhito").unwrap();

        let mut hasher = Sha256::new();
        hasher.update([1]);
        hasher.update(b"yakuhito");
        assert_eq!(handle.hash(), Bytes32::new(hasher.finalize()));

        // The hash the mainnet registry stores this handle under.
        assert_eq!(
            handle.hash(),
            Bytes32::new(hex!(
                "dc41e4f945750385ea6e5e34e77f2713bfc47cb7ee207b24405374f32b36c24b"
            ))
        );
    }
}
