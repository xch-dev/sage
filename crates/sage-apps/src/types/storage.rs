use serde::{Deserialize, Deserializer, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SageAppStorage {
    AppleDataStore {
        identifier_hex: String,
    },
    WindowsProfile {
        #[serde(deserialize_with = "deserialize_profile_directory_name")]
        directory_name: String,
    },
    Unmanaged,
}

/// Validates a persisted Windows profile directory name before it is joined
/// onto Sage's app-data directory or passed to storage cleanup.
pub(crate) fn validate_profile_directory_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("profile directory name must not be empty".to_string());
    }

    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
    {
        return Err(format!("invalid profile directory name: {name}"));
    }

    Ok(())
}

fn deserialize_profile_directory_name<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let name = String::deserialize(deserializer)?;
    validate_profile_directory_name(&name).map_err(serde::de::Error::custom)?;
    Ok(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn windows_profile_json(directory_name: &str) -> String {
        let serialized = serde_json::to_string(&SageAppStorage::WindowsProfile {
            directory_name: "PLACEHOLDER".to_string(),
        })
        .unwrap();
        let encoded_name = serde_json::to_string(directory_name).unwrap();

        serialized.replace("\"PLACEHOLDER\"", &encoded_name)
    }

    #[test]
    fn windows_profile_accepts_expected_directory_names() {
        for name in [
            "profile-8c8532e1-14d8-4d5f-b652-4ff29fc35a19",
            "builtin-profile-__sage_test_storage_isolation_persistent",
        ] {
            let storage: SageAppStorage =
                serde_json::from_str(&windows_profile_json(name)).unwrap();
            assert_eq!(
                storage,
                SageAppStorage::WindowsProfile {
                    directory_name: name.to_string()
                }
            );
        }
    }

    #[test]
    fn windows_profile_rejects_traversal_and_separator_directory_names() {
        for name in [
            "..",
            "../evil",
            "..\\evil",
            "C:\\evil",
            "profiles/../..",
            "a/b",
            ".",
            "profile-abc.def",
            "",
        ] {
            assert!(
                serde_json::from_str::<SageAppStorage>(&windows_profile_json(name)).is_err(),
                "expected directory name {name:?} to be rejected"
            );
        }
    }
}
