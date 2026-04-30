use crate::types::{InstalledSageAppStorage, PendingStorageCleanupTarget};

#[cfg(target_os = "windows")]
pub fn data_directory_for(directory_name: &str) -> PathBuf {
    PathBuf::from("profiles").join(directory_name)
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub fn parse_data_store_id(identifier_hex: &str) -> Result<[u8; 16], String> {
    let bytes = hex::decode(identifier_hex)
        .map_err(|err| format!("invalid data store identifier hex: {err}"))?;

    if bytes.len() != 16 {
        return Err(format!(
            "invalid data store identifier length {}, expected 16 bytes",
            bytes.len()
        ));
    }

    let mut out = [0_u8; 16];
    out.copy_from_slice(&bytes);
    Ok(out)
}

pub fn cleanup_target_from_storage(
    storage: &InstalledSageAppStorage,
) -> PendingStorageCleanupTarget {
    match storage {
        InstalledSageAppStorage::AppleDataStore { identifier_hex } => {
            PendingStorageCleanupTarget::AppleDataStore {
                identifier_hex: identifier_hex.clone(),
            }
        }
        InstalledSageAppStorage::WindowsProfile { directory_name } => {
            PendingStorageCleanupTarget::WindowsProfile {
                directory_name: directory_name.clone(),
            }
        }
        InstalledSageAppStorage::Unmanaged => PendingStorageCleanupTarget::Unmanaged,
    }
}
