use crate::{Error, Result, Sage, parse_nft_id};
use sage_api::{
    DeleteUserTheme, DeleteUserThemeResponse, GetNftData, GetUserTheme, GetUserThemeResponse,
    GetUserThemes, GetUserThemesResponse, SaveUserTheme, SaveUserThemeResponse,
};
use serde_json::Value;
use std::{io::ErrorKind, path::PathBuf};
use tokio::fs;

impl Sage {
    async fn checked_themes_root(&self) -> Result<PathBuf> {
        let root = self.path.join("themes");
        match fs::symlink_metadata(&root).await {
            Ok(metadata) if metadata.file_type().is_symlink() => Err(Error::InvalidThemeJson),
            Ok(_) => Ok(fs::canonicalize(root).await?),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(root),
            Err(error) => Err(error.into()),
        }
    }

    /// All theme endpoints share NFT validation and filesystem containment checks.
    async fn checked_theme_directory(&self, nft_id: &str) -> Result<PathBuf> {
        parse_nft_id(nft_id.to_owned())?;
        let root = self.checked_themes_root().await?;
        let directory = root.join(nft_id);
        for path in [&directory, &directory.join("theme.json")] {
            match fs::symlink_metadata(path).await {
                Ok(_) => {
                    if !fs::canonicalize(path).await?.starts_with(&root) {
                        return Err(Error::InvalidNftId(nft_id.to_owned()));
                    }
                }
                Err(error) if error.kind() == ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        }
        Ok(directory)
    }

    pub async fn delete_user_theme(&self, req: DeleteUserTheme) -> Result<DeleteUserThemeResponse> {
        if req.nft_id.is_empty() {
            return Ok(DeleteUserThemeResponse {});
        }

        let theme_dir = self.checked_theme_directory(&req.nft_id).await?;

        if !theme_dir.exists() {
            return Ok(DeleteUserThemeResponse {});
        }

        fs::remove_dir_all(&theme_dir).await?;

        Ok(DeleteUserThemeResponse {})
    }

    pub async fn get_user_theme(&self, req: GetUserTheme) -> Result<GetUserThemeResponse> {
        if req.nft_id.is_empty() {
            return Ok(GetUserThemeResponse { theme: None });
        }

        let theme_dir = self.checked_theme_directory(&req.nft_id).await?;
        let theme_json_path = theme_dir.join("theme.json");

        if !theme_json_path.exists() {
            return Ok(GetUserThemeResponse { theme: None });
        }

        let theme_json = fs::read_to_string(&theme_json_path).await?;

        Ok(GetUserThemeResponse {
            theme: Some(theme_json),
        })
    }

    pub async fn save_user_theme(&self, req: SaveUserTheme) -> Result<SaveUserThemeResponse> {
        if req.nft_id.is_empty() {
            return Ok(SaveUserThemeResponse {});
        }

        let nft_theme_dir = self.checked_theme_directory(&req.nft_id).await?;
        let themes_dir = self.checked_themes_root().await?;

        if !themes_dir.exists() {
            fs::create_dir_all(&themes_dir).await?;
        }

        if !nft_theme_dir.exists() {
            fs::create_dir_all(&nft_theme_dir).await?;
        }

        // Get NFT data to extract the theme JSON
        let nft_data_response = self
            .get_nft_data(GetNftData {
                nft_id: req.nft_id.clone(),
            })
            .await?;

        let theme_json_path = nft_theme_dir.join("theme.json");

        if let Some(nft_data) = nft_data_response.data
            && let Some(metadata_json) = nft_data.metadata_json
        {
            let json_value: Value =
                serde_json::from_str(&metadata_json).map_err(|_| crate::Error::InvalidThemeJson)?;

            // Extract the data.theme node
            let mut theme_data = json_value
                .get("data")
                .and_then(|data| data.get("theme"))
                .ok_or(crate::Error::MissingThemeData)?
                .clone();

            // Set the name property to the NFT ID
            if let Some(theme_obj) = theme_data.as_object_mut() {
                theme_obj.insert(
                    "name".to_string(),
                    serde_json::Value::String(req.nft_id.clone()),
                );
            }

            let theme_json = serde_json::to_string_pretty(&theme_data)
                .map_err(|_| crate::Error::InvalidThemeJson)?;
            fs::write(&theme_json_path, theme_json).await?;
        }

        Ok(SaveUserThemeResponse {})
    }

    pub async fn get_user_themes(&self, _req: GetUserThemes) -> Result<GetUserThemesResponse> {
        let themes_dir = self.checked_themes_root().await?;
        let mut themes = Vec::new();

        if !themes_dir.exists() {
            return Ok(GetUserThemesResponse { themes });
        }

        match fs::read_dir(&themes_dir).await {
            Ok(mut entries) => {
                while let Some(entry) = entries.next_entry().await? {
                    let name = entry.file_name();
                    let Some(nft_id) = name.to_str() else {
                        continue;
                    };
                    let Ok(path) = self.checked_theme_directory(nft_id).await else {
                        continue;
                    };

                    if path.is_dir() {
                        let theme_json_path = path.join("theme.json");

                        if theme_json_path.exists() {
                            match fs::read_to_string(&theme_json_path).await {
                                Ok(theme_content) => {
                                    themes.push(theme_content);
                                }
                                Err(e) => {
                                    eprintln!(
                                        "Failed to read theme.json in {}: {e}",
                                        path.display()
                                    );
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read themes directory: {e}");
            }
        }

        Ok(GetUserThemesResponse { themes })
    }
}
