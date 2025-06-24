use chia::protocol::Bytes32;
use chia_sha2::Sha256;
use sage_assets::{Chip0007Metadata, Collection};
use sage_database::CollectionRow;
use tracing::warn;

#[derive(Debug, Default, Clone)]
pub struct ComputedNftInfo {
    pub name: Option<String>,
    pub description: Option<String>,
    pub sensitive_content: bool,
    pub collection: Option<CollectionRow>,
}

pub fn compute_nft_info(did_id: Option<Bytes32>, blob: &[u8]) -> ComputedNftInfo {
    let Some(json) = Chip0007Metadata::from_bytes(blob)
        .map_err(|error| {
            warn!(
                "Error parsing offchain metadata: {error}, {}",
                String::from_utf8_lossy(blob)
            );
            error
        })
        .ok()
    else {
        return ComputedNftInfo::default();
    };

    let sensitive_content = json.is_sensitive();

    let collection = if let (
        Some(did_id),
        Some(Collection {
            id: metadata_collection_id,
            name,
            attributes,
        }),
    ) = (did_id, json.collection)
    {
        let attributes = attributes.unwrap_or_default();
        Some(CollectionRow {
            description: None,
            created_height: None,
            hash: calculate_collection_id(did_id, &metadata_collection_id.to_string()),
            minter_hash: did_id,
            uuid: metadata_collection_id.to_string(),
            name: Some(name),
            icon_url: attributes.iter().find_map(|item| {
                match (item.kind.as_str(), item.value.as_str()) {
                    (Some("icon"), Some(icon)) => Some(icon.to_string()),
                    _ => None,
                }
            }),
            banner_url: attributes.iter().find_map(|item| {
                match (item.kind.as_str(), item.value.as_str()) {
                    (Some("banner"), Some(banner)) => Some(banner.to_string()),
                    _ => None,
                }
            }),
            is_visible: true,
        })
    } else {
        None
    };

    ComputedNftInfo {
        name: Some(json.name.clone()),
        description: Some(json.description.clone()),
        sensitive_content,
        collection,
    }
}

fn calculate_collection_id(did_id: Bytes32, json_collection_id: &str) -> Bytes32 {
    let mut hasher = Sha256::new();
    hasher.update(hex::encode(did_id));
    hasher.update(json_collection_id);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::*;

    #[test]
    fn test_calculate_collection_id() {
        let did_id = Bytes32::new(hex!(
            "6b6ad6846c3341dbdb627fcc6cd069d58eac16b3e632d3812d895a9d0f9d3914"
        ));
        let json_collection_id = "add5c821-296b-4338-9c88-33d8402cac56";
        let collection_id = calculate_collection_id(did_id, json_collection_id);
        assert_eq!(
            collection_id,
            Bytes32::new(hex!(
                "d28b49d9f69f4d06471b58ab4524dcc40a70e0eab6030548cff716a092785f24"
            ))
        );
    }
}
