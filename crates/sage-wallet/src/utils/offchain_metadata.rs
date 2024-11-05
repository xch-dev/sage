use chia::{protocol::Bytes32, sha2::Sha256};
use sage_database::CollectionRow;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OffchainMetadata {
    #[serde(default)]
    name: Option<String>,

    #[serde(default)]
    sensitive_content: Option<Value>,

    #[serde(default)]
    collection: Option<Collection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Collection {
    #[serde(default)]
    id: Option<String>,

    #[serde(default)]
    name: Option<String>,

    #[serde(default)]
    attributes: Option<Vec<Attribute>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Attribute {
    #[serde(default, rename = "type")]
    key: Option<String>,

    #[serde(default)]
    value: Option<Value>,
}

#[derive(Debug, Default, Clone)]
pub struct ComputedNftInfo {
    pub name: Option<String>,
    pub sensitive_content: bool,
    pub collection: Option<CollectionRow>,
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
            attributes,
        }),
    ) = (did_id, json.collection)
    {
        Some(CollectionRow {
            collection_id: calculate_collection_id(did_id, &metadata_collection_id),
            did_id,
            metadata_collection_id,
            name,
            icon: attributes.unwrap_or_default().into_iter().find_map(|item| {
                match (item.key.as_deref(), item.value) {
                    (Some("icon"), Some(Value::String(icon))) => Some(icon),
                    _ => None,
                }
            }),
            visible: true,
        })
    } else {
        None
    };

    ComputedNftInfo {
        name: json.name,
        sensitive_content: json.sensitive_content.is_some_and(|value| match value {
            Value::Bool(value) => value,
            Value::Null => false,
            Value::Array(items) => !items.is_empty(),
            Value::Object(items) => !items.is_empty(),
            Value::Number(_value) => true,
            Value::String(value) => !value.is_empty(),
        }),
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
