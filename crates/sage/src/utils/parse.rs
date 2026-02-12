#![allow(clippy::needless_pass_by_value)]

use chia_wallet_sdk::prelude::*;
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

pub fn parse_coin_ids(input: Vec<String>) -> Result<Vec<Bytes32>> {
    input.into_iter().map(parse_coin_id).collect()
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

pub fn parse_option_id(input: String) -> Result<Bytes32> {
    let address = Address::decode(&input)?;

    if address.prefix != "option" {
        return Err(Error::InvalidOptionId(input));
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

pub fn parse_memos(input: Vec<String>) -> Result<Vec<Bytes>> {
    let mut memos = Vec::new();
    for memo in input {
        memos.push(Bytes::from(hex::decode(memo)?));
    }
    Ok(memos)
}

pub fn parse_any_asset_id(input: String) -> Result<Bytes32> {
    Ok(if input.starts_with("nft") {
        parse_nft_id(input)?
    } else if input.starts_with("did:chia:") {
        parse_did_id(input)?
    } else if input.starts_with("option") {
        parse_option_id(input)?
    } else {
        // Assume it's a CAT token (hex string)
        parse_asset_id(input)?
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    // ─── parse_signature_message ───

    #[test]
    fn test_parse_signature_message_hex_with_prefix() {
        let input = "0x1234567890abcdef";
        let result = parse_signature_message(input.to_string()).unwrap();
        let expected = Bytes::from(hex::decode(&input[2..]).unwrap());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_signature_message_hex_without_prefix() {
        let input = "1234567890abcdef";
        let result = parse_signature_message(input.to_string()).unwrap();
        let expected = Bytes::from(hex::decode(input).unwrap());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_signature_message_non_hex() {
        let input = "Hello, world!";
        let result = parse_signature_message(input.to_string()).unwrap();
        let expected = Bytes::from(input.as_bytes());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_signature_message_short_hex() {
        let input = "cafe";
        let result = parse_signature_message(input.to_string()).unwrap();
        let expected = Bytes::from(hex::decode(input).unwrap());
        assert_eq!(result, expected);
    }

    // ─── parse_asset_id ───

    #[test]
    fn test_parse_asset_id_valid() {
        let hex_str = "aa".repeat(32); // 64 hex chars = 32 bytes
        let result = parse_asset_id(hex_str.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Bytes32::new([0xaa; 32]));
    }

    #[test]
    fn test_parse_asset_id_invalid_length() {
        let hex_str = "aa".repeat(16); // too short
        let result = parse_asset_id(hex_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_asset_id_invalid_hex() {
        let input = "zz".repeat(32);
        let result = parse_asset_id(input);
        assert!(result.is_err());
    }

    // ─── parse_coin_id ───

    #[test]
    fn test_parse_coin_id_without_prefix() {
        let hex_str = "bb".repeat(32);
        let result = parse_coin_id(hex_str);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Bytes32::new([0xbb; 32]));
    }

    #[test]
    fn test_parse_coin_id_with_0x_prefix() {
        let hex_str = format!("0x{}", "cc".repeat(32));
        let result = parse_coin_id(hex_str);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Bytes32::new([0xcc; 32]));
    }

    #[test]
    fn test_parse_coin_id_invalid() {
        let result = parse_coin_id("not_hex".to_string());
        assert!(result.is_err());
    }

    // ─── parse_coin_ids ───

    #[test]
    fn test_parse_coin_ids_multiple() {
        let ids = vec!["aa".repeat(32), format!("0x{}", "bb".repeat(32))];
        let result = parse_coin_ids(ids);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_parse_coin_ids_one_invalid() {
        let ids = vec!["aa".repeat(32), "invalid".to_string()];
        let result = parse_coin_ids(ids);
        assert!(result.is_err());
    }

    // ─── parse_hash ───

    #[test]
    fn test_parse_hash_valid() {
        let hex_str = "dd".repeat(32);
        let result = parse_hash(hex_str);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_hash_with_0x() {
        let hex_str = format!("0x{}", "ee".repeat(32));
        let result = parse_hash(hex_str);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_hash_invalid_length() {
        let result = parse_hash("abcd".to_string());
        assert!(result.is_err());
    }

    // ─── parse_amount ───

    #[test]
    fn test_parse_amount_valid_integer() {
        let amount: Amount = serde_json::from_str("1000").unwrap();
        let result = parse_amount(amount);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1000);
    }

    #[test]
    fn test_parse_amount_zero() {
        let amount: Amount = serde_json::from_str("0").unwrap();
        let result = parse_amount(amount);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    // ─── parse_program ───

    #[test]
    fn test_parse_program_valid() {
        let result = parse_program("ff01ff02".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_program_with_0x() {
        let result = parse_program("0xff01".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_program_invalid_hex() {
        let result = parse_program("zzz".to_string());
        assert!(result.is_err());
    }

    // ─── parse_memos ───

    #[test]
    fn test_parse_memos_valid() {
        let memos = vec!["aabb".to_string(), "ccdd".to_string()];
        let result = parse_memos(memos);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_parse_memos_empty() {
        let result = parse_memos(vec![]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_parse_memos_invalid_hex() {
        let memos = vec!["not_hex".to_string()];
        let result = parse_memos(memos);
        assert!(result.is_err());
    }

    // ─── parse_any_asset_id routing ───

    #[test]
    fn test_parse_any_asset_id_hex() {
        let hex_str = "aa".repeat(32);
        let result = parse_any_asset_id(hex_str);
        assert!(result.is_ok());
    }
}
