#![allow(clippy::needless_pass_by_value)]

use chia::{
    bls::{PublicKey, Signature},
    protocol::{Bytes, Bytes32, Program},
};
use chia_wallet_sdk::utils::Address;
use sage_api::Amount;

use crate::{Error, Result};

pub fn parse_asset_id(input: String) -> Result<Bytes32> {
    let asset_id: [u8; 32] = hex::decode(&input)?
        .try_into()
        .map_err(|_| Error::InvalidAssetId(input))?;
    Ok(asset_id.into())
}

pub fn parse_coin_id(input: String) -> Result<Bytes32> {
    let stripped = if let Some(stripped) = input.strip_prefix("0x") {
        stripped
    } else {
        &input
    };

    let asset_id: [u8; 32] = hex::decode(stripped)?
        .try_into()
        .map_err(|_| Error::InvalidCoinId(input))?;
    Ok(asset_id.into())
}

pub fn parse_did_id(input: String) -> Result<Bytes32> {
    let address = Address::decode(&input)?;

    if address.prefix != "did:chia:" {
        return Err(Error::InvalidDidId(input));
    }

    Ok(address.puzzle_hash)
}

pub fn parse_nft_id(input: String) -> Result<Bytes32> {
    let address = Address::decode(&input)?;

    if address.prefix != "nft" {
        return Err(Error::InvalidNftId(input));
    }

    Ok(address.puzzle_hash)
}

pub fn parse_collection_id(input: String) -> Result<Bytes32> {
    let address = Address::decode(&input)?;

    if address.prefix != "col" {
        return Err(Error::InvalidCollectionId(input));
    }

    Ok(address.puzzle_hash)
}

pub fn parse_offer_id(input: String) -> Result<Bytes32> {
    let asset_id: [u8; 32] = hex::decode(&input)?
        .try_into()
        .map_err(|_| Error::InvalidOfferId(input))?;
    Ok(asset_id.into())
}

pub fn parse_amount(input: Amount) -> Result<u64> {
    let Some(amount) = input.to_u64() else {
        return Err(Error::InvalidAmount(input.to_string()));
    };
    Ok(amount)
}

pub fn parse_hash(input: String) -> Result<Bytes32> {
    let stripped = if let Some(stripped) = input.strip_prefix("0x") {
        stripped
    } else {
        &input
    };

    hex::decode(stripped)?
        .try_into()
        .map_err(|_| Error::InvalidHash(input))
}

pub fn parse_signature(input: String) -> Result<Signature> {
    let stripped = if let Some(stripped) = input.strip_prefix("0x") {
        stripped
    } else {
        &input
    };

    let signature: [u8; 96] = hex::decode(stripped)?
        .try_into()
        .map_err(|_| Error::InvalidSignature(input))?;

    Ok(Signature::from_bytes(&signature)?)
}

/// Parse a signature message.
///
/// It takes a string and returns a Bytes object.
///
/// This function supports hex strings with or without a 0x prefix.
/// It also supports non-hex strings.
pub fn parse_signature_message(input: String) -> Result<Bytes> {
    let stripped = if let Some(stripped) = input.strip_prefix("0x") {
        stripped
    } else {
        &input
    };

    if stripped.chars().all(|c| c.is_ascii_hexdigit()) && !stripped.is_empty() {
        Ok(Bytes::from(hex::decode(stripped)?))
    } else {
        Ok(Bytes::from(input.as_bytes()))
    }
}

pub fn parse_public_key(input: String) -> Result<PublicKey> {
    let stripped = if let Some(stripped) = input.strip_prefix("0x") {
        stripped
    } else {
        &input
    };

    let public_key: [u8; 48] = hex::decode(stripped)?
        .try_into()
        .map_err(|_| Error::InvalidPublicKey(input))?;

    Ok(PublicKey::from_bytes(&public_key)?)
}

pub fn parse_program(input: String) -> Result<Program> {
    let stripped = if let Some(stripped) = input.strip_prefix("0x") {
        stripped
    } else {
        &input
    };

    Ok(hex::decode(stripped)?.into())
}

pub fn parse_memos(input: Option<Vec<String>>) -> Result<Option<Vec<Bytes>>> {
    if let Some(list) = input {
        let mut memos = Vec::new();
        for memo in list {
            memos.push(Bytes::from(hex::decode(memo)?));
        }
        Ok(Some(memos))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn test_parse_signature_message() {
        // Test hex string with 0x prefix
        let input = "0x1234567890abcdef";
        let result = parse_signature_message(input.to_string()).unwrap();
        let expected = Bytes::from(hex::decode(&input[2..]).unwrap());
        assert_eq!(result, expected);

        // Test hex string without prefix
        let input = "1234567890abcdef";
        let result = parse_signature_message(input.to_string()).unwrap();
        let expected = Bytes::from(hex::decode(input).unwrap());
        assert_eq!(result, expected);

        // Test non-hex string
        let input = "Hello, world!";
        let result = parse_signature_message(input.to_string()).unwrap();
        let expected = Bytes::from(input.as_bytes());
        assert_eq!(result, expected);

        // Test hex string with 0x prefix
        let input = "0xcafe";
        let result = parse_signature_message(input.to_string()).unwrap();
        let expected = Bytes::from(hex::decode(&input[2..]).unwrap());
        assert_eq!(result, expected);

        // Test hex string without prefix
        let input = "cafe";
        let result = parse_signature_message(input.to_string()).unwrap();
        let expected = Bytes::from(hex::decode(input).unwrap());
        assert_eq!(result, expected);
    }
}
