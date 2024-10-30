use chia::{protocol::Bytes32, sha2::Sha256};
use sage_database::NftCollectionRow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OffchainMetadata {
    #[serde(default)]
    name: Option<String>,

    #[serde(default)]
    collection: Option<Collection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Collection {
    #[serde(default)]
    id: Option<String>,

    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct ComputedNftInfo {
    pub name: Option<String>,
    pub collection: Option<NftCollectionRow>,
}

pub fn compute_nft_info(did_id: Option<Bytes32>, blob: Option<&[u8]>) -> ComputedNftInfo {
    let Some(json) = offchain_metadata(blob) else {
        return ComputedNftInfo::default();
    };

    let collection = if let (
        Some(did_id),
        Some(Collection {
            id: Some(metadata_collection_id),
            name,
        }),
    ) = (did_id, json.collection)
    {
        Some(NftCollectionRow {
            collection_id: calculate_collection_id(did_id, &metadata_collection_id),
            did_id,
            metadata_collection_id,
            name,
        })
    } else {
        None
    };

    ComputedNftInfo {
        name: json.name,
        collection,
    }
}

fn offchain_metadata(blob: Option<&[u8]>) -> Option<OffchainMetadata> {
    serde_json::from_slice(blob?).ok()
}

fn calculate_collection_id(did_id: Bytes32, json_collection_id: &str) -> Bytes32 {
    let mut hasher = Sha256::new();
    hasher.update(did_id);
    hasher.update(json_collection_id);
    hasher.finalize().into()
}
