use crate::{Result, Sage};
use sage_api::{GetUserThemes, GetUserThemesResponse};
use tokio::fs;

impl Sage {
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
