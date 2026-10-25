use chia_wallet_sdk::{
    chia::puzzle_types::nft::NftMetadata,
    prelude::*,
    puzzles::NFT_METADATA_UPDATER_DEFAULT_HASH,
    types::puzzles::{ANY_METADATA_UPDATER_HASH, HandleNftMetadata},
};

use crate::{Convert, DatabaseError, Result};

/// The maximum number of characters kept from an on-chain display name.
const MAX_DISPLAY_NAME_CHARS: usize = 128;

/// The kind of metadata updater puzzle an NFT uses, which determines how its
/// on-chain metadata can change over time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NftUpdaterKind {
    /// The standard CHIP-7 updater, which can only append URIs.
    Default = 0,
    /// The updater used by XCHandles, which lets the owner replace all metadata.
    Any = 1,
    /// An updater Sage doesn't recognize, so nothing is known about its behavior.
    Unknown = 2,
}

impl NftUpdaterKind {
    pub fn from_puzzle_hash(puzzle_hash: Bytes32) -> Self {
        if puzzle_hash == NFT_METADATA_UPDATER_DEFAULT_HASH.into() {
            Self::Default
        } else if puzzle_hash == ANY_METADATA_UPDATER_HASH.into() {
            Self::Any
        } else {
            Self::Unknown
        }
    }

    /// Whether the metadata can only change by appending URIs.
    pub fn is_append_only(self) -> bool {
        matches!(self, Self::Default)
    }
}

impl Convert<NftUpdaterKind> for i64 {
    fn convert(self) -> Result<NftUpdaterKind> {
        Ok(match self {
            0 => NftUpdaterKind::Default,
            1 => NftUpdaterKind::Any,
            2 => NftUpdaterKind::Unknown,
            _ => return Err(DatabaseError::InvalidEnumVariant),
        })
    }
}

/// On-chain NFT metadata, parsed according to the NFT's metadata updater.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedNftMetadata {
    /// The `dn` field of XCHandles metadata. This is owner-editable display data
    /// and a lookup hint only, never proof of handle ownership.
    pub display_name: Option<String>,
    pub data_uris: Vec<String>,
    pub data_hash: Option<Bytes32>,
    pub metadata_uris: Vec<String>,
    pub metadata_hash: Option<Bytes32>,
    pub license_uris: Vec<String>,
    pub license_hash: Option<Bytes32>,
    pub edition_number: Option<u64>,
    pub edition_total: Option<u64>,
}

impl ParsedNftMetadata {
    /// Parses the metadata as `HandleNftMetadata` for NFTs using the any metadata updater,
    /// and as CHIP-7 `NftMetadata` otherwise. If handle metadata is malformed, it falls
    /// back to CHIP-7 parsing so the NFT can still be displayed.
    pub fn parse(allocator: &Allocator, ptr: NodePtr, kind: NftUpdaterKind) -> Option<Self> {
        if kind == NftUpdaterKind::Any
            && let Ok(metadata) = HandleNftMetadata::from_clvm(allocator, ptr)
        {
            return Some(metadata.into());
        }

        NftMetadata::from_clvm(allocator, ptr).ok().map(Into::into)
    }

    pub fn parse_program(program: &Program, kind: NftUpdaterKind) -> Option<Self> {
        let mut allocator = Allocator::new();
        let ptr = program.to_clvm(&mut allocator).ok()?;
        Self::parse(&allocator, ptr, kind)
    }
}

impl From<NftMetadata> for ParsedNftMetadata {
    fn from(metadata: NftMetadata) -> Self {
        Self {
            display_name: None,
            data_uris: metadata.data_uris,
            data_hash: metadata.data_hash,
            metadata_uris: metadata.metadata_uris,
            metadata_hash: metadata.metadata_hash,
            license_uris: metadata.license_uris,
            license_hash: metadata.license_hash,
            edition_number: Some(metadata.edition_number),
            edition_total: Some(metadata.edition_total),
        }
    }
}

impl From<HandleNftMetadata> for ParsedNftMetadata {
    fn from(metadata: HandleNftMetadata) -> Self {
        Self {
            display_name: metadata
                .display_name
                .as_deref()
                .and_then(sanitize_display_name),
            data_uris: metadata.image_uris,
            data_hash: metadata.image_hash,
            metadata_uris: metadata.metadata_uris,
            metadata_hash: metadata.metadata_hash,
            license_uris: metadata.license_uris,
            license_hash: metadata.license_hash,
            edition_number: None,
            edition_total: None,
        }
    }
}

/// Removes control and bidirectional formatting characters, which could be used to make
/// a display name look like something it isn't, and caps its length.
fn sanitize_display_name(display_name: &str) -> Option<String> {
    let sanitized: String = display_name
        .chars()
        .filter(|&c| !c.is_control() && !is_bidi_control(c))
        .take(MAX_DISPLAY_NAME_CHARS)
        .collect();

    let trimmed = sanitized.trim();

    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn is_bidi_control(c: char) -> bool {
    matches!(
        c,
        '\u{061C}' | '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}'
    )
}

#[cfg(test)]
mod tests {
    use chia_wallet_sdk::clvmr::serde::node_from_bytes;
    use hex_literal::hex;

    use super::*;

    /// The on-chain metadata of the `@yakuhito` XCHandles NFT on mainnet
    /// (launcher `dfc272581333cf5b1b3b584d216966e4f543fb1bfc2b6ef3a78b389f18b7a37e`).
    const YAKUHITO_METADATA: [u8; 298] = hex!(
        "
        ffff82646e894079616b756869746fffff75ffaa68747470733a2f2f6e6674732e
        786368616e646c65732e636f6d2f76312f79616b756869746f2e706e6780ffff68
        a037881600d0e7602bfc8fedb58e2daecee40b9898ca586e50116bceabb5f99a51
        ffff826d75ffab68747470733a2f2f6e6674732e786368616e646c65732e636f6d
        2f76312f79616b756869746f2e6a736f6e80ffff826d68a099ac450670fff1890d
        767a68e20923839df1f75cd08f955427e15bf1604e4b20ffff826c75ffbf687474
        70733a2f2f6372656174697665636f6d6d6f6e732e6f72672f7075626c6963646f
        6d61696e2f7a65726f2f312e302f6c6567616c636f64652e74787480ffff826c68
        a0a2010f343487d3f7618affe54f789f5487602331c0a8d03f49e9a7c547cf0499
        80
        "
    );

    fn parse_bytes(bytes: &[u8], kind: NftUpdaterKind) -> Option<ParsedNftMetadata> {
        let mut allocator = Allocator::new();
        let ptr = node_from_bytes(&mut allocator, bytes).unwrap();
        ParsedNftMetadata::parse(&allocator, ptr, kind)
    }

    fn encode<T: ToClvm<Allocator>>(value: &T) -> Program {
        let mut allocator = Allocator::new();
        let ptr = value.to_clvm(&mut allocator).unwrap();
        Program::from_clvm(&allocator, ptr).unwrap()
    }

    #[test]
    fn classifies_updater_puzzle_hashes() {
        assert_eq!(
            NftUpdaterKind::from_puzzle_hash(NFT_METADATA_UPDATER_DEFAULT_HASH.into()),
            NftUpdaterKind::Default
        );
        assert_eq!(
            NftUpdaterKind::from_puzzle_hash(ANY_METADATA_UPDATER_HASH.into()),
            NftUpdaterKind::Any
        );
        assert_eq!(
            NftUpdaterKind::from_puzzle_hash(Bytes32::new([7; 32])),
            NftUpdaterKind::Unknown
        );
        assert!(NftUpdaterKind::Default.is_append_only());
        assert!(!NftUpdaterKind::Any.is_append_only());
        assert!(!NftUpdaterKind::Unknown.is_append_only());
    }

    #[test]
    fn parses_mainnet_handle_metadata() {
        let metadata = parse_bytes(&YAKUHITO_METADATA, NftUpdaterKind::Any).unwrap();

        assert_eq!(metadata.display_name.as_deref(), Some("@yakuhito"));
        assert_eq!(
            metadata.data_uris,
            ["https://nfts.xchandles.com/v1/yakuhito.png"]
        );
        assert_eq!(
            metadata.data_hash,
            Some(Bytes32::new(hex!(
                "37881600d0e7602bfc8fedb58e2daecee40b9898ca586e50116bceabb5f99a51"
            )))
        );
        assert_eq!(
            metadata.metadata_uris,
            ["https://nfts.xchandles.com/v1/yakuhito.json"]
        );
        assert_eq!(
            metadata.metadata_hash,
            Some(Bytes32::new(hex!(
                "99ac450670fff1890d767a68e20923839df1f75cd08f955427e15bf1604e4b20"
            )))
        );
        assert_eq!(metadata.license_uris.len(), 1);
        assert!(metadata.license_hash.is_some());
        assert_eq!(metadata.edition_number, None);
        assert_eq!(metadata.edition_total, None);
    }

    #[test]
    fn ignores_display_name_for_default_updater() {
        let metadata = parse_bytes(&YAKUHITO_METADATA, NftUpdaterKind::Default).unwrap();

        assert_eq!(metadata.display_name, None);
        assert_eq!(
            metadata.data_uris,
            ["https://nfts.xchandles.com/v1/yakuhito.png"]
        );
        assert_eq!(metadata.edition_number, Some(1));
        assert_eq!(metadata.edition_total, Some(1));
    }

    #[test]
    fn parses_chip7_metadata() {
        let program = encode(&NftMetadata {
            edition_number: 3,
            edition_total: 10,
            data_uris: vec!["https://example.com/a.png".to_string()],
            data_hash: Some(Bytes32::new([1; 32])),
            metadata_uris: vec!["https://example.com/a.json".to_string()],
            metadata_hash: Some(Bytes32::new([2; 32])),
            license_uris: vec![],
            license_hash: None,
        });

        for kind in [NftUpdaterKind::Default, NftUpdaterKind::Unknown] {
            let metadata = ParsedNftMetadata::parse_program(&program, kind).unwrap();
            assert_eq!(metadata.display_name, None);
            assert_eq!(metadata.data_uris, ["https://example.com/a.png"]);
            assert_eq!(metadata.data_hash, Some(Bytes32::new([1; 32])));
            assert_eq!(metadata.metadata_hash, Some(Bytes32::new([2; 32])));
            assert_eq!(metadata.edition_number, Some(3));
            assert_eq!(metadata.edition_total, Some(10));
        }
    }

    #[test]
    fn malformed_handle_metadata_falls_back_to_chip7() {
        // `dn` is a list rather than a string, which `HandleNftMetadata` rejects.
        let program = encode(&vec![
            ("dn", encode(&vec!["not", "a", "string"])),
            ("u", encode(&vec!["https://example.com/a.png"])),
            ("h", encode(&Bytes32::new([1; 32]))),
        ]);

        let metadata = ParsedNftMetadata::parse_program(&program, NftUpdaterKind::Any).unwrap();
        assert_eq!(metadata.display_name, None);
        assert_eq!(metadata.data_uris, ["https://example.com/a.png"]);
        assert_eq!(metadata.data_hash, Some(Bytes32::new([1; 32])));

        // A data hash with the wrong length is rejected by `HandleNftMetadata`, and
        // ignored by the lenient CHIP-7 parser.
        let program = encode(&vec![
            ("dn", encode(&"@alice")),
            ("h", encode(&vec![1u8, 2, 3])),
        ]);

        let metadata = ParsedNftMetadata::parse_program(&program, NftUpdaterKind::Any).unwrap();
        assert_eq!(metadata.display_name, None);
        assert_eq!(metadata.data_hash, None);

        // Metadata that isn't a key value list at all can't be parsed.
        let program = encode(&"garbage");
        assert_eq!(
            ParsedNftMetadata::parse_program(&program, NftUpdaterKind::Any),
            None
        );
    }

    #[test]
    fn empty_handle_metadata_parses() {
        let metadata = parse_bytes(&hex!("80"), NftUpdaterKind::Any).unwrap();
        assert_eq!(metadata.display_name, None);
        assert!(metadata.data_uris.is_empty());
        assert_eq!(metadata.data_hash, None);
    }

    #[test]
    fn sanitizes_display_names() {
        assert_eq!(
            sanitize_display_name("@ali\u{202E}ce\n").as_deref(),
            Some("@alice")
        );
        assert_eq!(sanitize_display_name("  \u{200F} "), None);
        assert_eq!(
            sanitize_display_name(&"a".repeat(500)).map(|name| name.chars().count()),
            Some(MAX_DISPLAY_NAME_CHARS)
        );
    }
}
