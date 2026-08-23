use serde::{Deserialize, Serialize};

pub(crate) fn is_sage_app_webview_label(label: &str) -> bool {
    ["app-", "system-app-"]
        .iter()
        .any(|prefix| label.strip_prefix(prefix).is_some_and(|id| !id.is_empty()))
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IsNdefAvailableRequest {}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IsNdefAvailableResponse {
    pub available: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetNdefPayloadsRequest {}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetNdefPayloadsResponse {
    pub payloads: Vec<Vec<u8>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetWebviewBoundsRequest {
    pub label: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetWebviewBoundsResponse {}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotWebviewRequest {
    pub label: String,
    pub width: f64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotWebviewResponse {
    pub data_url: String,
}

#[cfg(test)]
mod tests {
    use super::is_sage_app_webview_label;

    #[test]
    fn accepts_only_sage_app_child_labels() {
        assert!(is_sage_app_webview_label("app-example"));
        assert!(is_sage_app_webview_label("system-app-donation"));

        assert!(!is_sage_app_webview_label("main"));
        assert!(!is_sage_app_webview_label("app-"));
        assert!(!is_sage_app_webview_label("system-app-"));
        assert!(!is_sage_app_webview_label("other-app-example"));
    }
}
