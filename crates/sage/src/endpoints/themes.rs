use crate::{Result, Sage};
use sage_api::{
    GetNftData, GetUserTheme, GetUserThemeResponse, GetUserThemes, GetUserThemesResponse,
    SaveUserTheme, SaveUserThemeResponse,
};
use serde_json::Value;
use tokio::fs;

impl Sage {
    pub async fn get_user_theme(&self, req: GetUserTheme) -> Result<GetUserThemeResponse> {
        let themes_dir = self.path.join("themes");
        let theme_json_path = themes_dir.join(&req.nft_id).join("theme.json");

        if !theme_json_path.exists() {
            return Ok(GetUserThemeResponse { theme: None });
        }

        let theme_json = fs::read_to_string(&theme_json_path).await?;

        Ok(GetUserThemeResponse {
            theme: Some(theme_json),
        })
    }

    pub async fn save_user_theme(&self, req: SaveUserTheme) -> Result<SaveUserThemeResponse> {
        let themes_dir = self.path.join("themes");

        // Create themes directory if it doesn't exist
        if !themes_dir.exists() {
            fs::create_dir_all(&themes_dir).await?;
        }

        // Create NFT-specific directory
        let nft_theme_dir = themes_dir.join(&req.nft_id);
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

        if let Some(nft_data) = nft_data_response.data {
            if let Some(metadata_json) = nft_data.metadata_json {
                // Parse the JSON and extract the data.theme node
                let json_value: Value = serde_json::from_str(&metadata_json)
                    .map_err(|_| crate::Error::InvalidThemeJson)?;

                // Extract the data.theme node
                let theme_data = json_value
                    .get("data")
                    .and_then(|data| data.get("theme"))
                    .ok_or(crate::Error::MissingThemeData)?;

                // Write the theme data to the file
                let theme_json = serde_json::to_string_pretty(theme_data)
                    .map_err(|_| crate::Error::InvalidThemeJson)?;
                fs::write(&theme_json_path, theme_json).await?;
            }
        }

        Ok(SaveUserThemeResponse {})
    }

    pub async fn get_user_themes(&self, _req: GetUserThemes) -> Result<GetUserThemesResponse> {
        let themes_dir = self.path.join("themes");
        let mut themes = Vec::new();

        if !themes_dir.exists() {
            return Ok(GetUserThemesResponse { themes });
        }

        match fs::read_dir(&themes_dir).await {
            Ok(mut entries) => {
                while let Some(entry) = entries.next_entry().await? {
                    let path = entry.path();

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
