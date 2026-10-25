use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_plugin_opener::OpenerExt;
use url::Url;

use crate::{
    BridgeApprovalRequestResult, BridgeContext, BridgeHandleResult, BridgeMethod,
    BridgeMethodCapability, BridgeMethodHandleError, BridgeTools, RustBridgeApprovalBody,
    RustBridgeApprovalRequest, RustBridgeRequest, UserBridgeCapability, parse_required_params,
};

const MAX_EXTERNAL_URL_LENGTH: usize = 2_048;

#[derive(Debug, Clone, Copy)]
pub struct EnvironmentOpenExternalUrl;

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentOpenExternalUrlParams {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct EnvironmentOpenExternalUrlResult {
    pub opened: bool,
}

fn validated_external_url(value: &str) -> Result<Url, BridgeMethodHandleError> {
    if value.len() > MAX_EXTERNAL_URL_LENGTH {
        return Err(BridgeMethodHandleError::invalid_request(format!(
            "url must not exceed {MAX_EXTERNAL_URL_LENGTH} bytes"
        )));
    }

    let url = Url::parse(value)
        .map_err(|err| BridgeMethodHandleError::invalid_request(format!("invalid url: {err}")))?;

    if !matches!(url.scheme(), "http" | "https") {
        return Err(BridgeMethodHandleError::invalid_request(
            "url scheme must be http or https",
        ));
    }

    if url.host_str().is_none() {
        return Err(BridgeMethodHandleError::invalid_request(
            "url must include a host",
        ));
    }

    if !url.username().is_empty() || url.password().is_some() {
        return Err(BridgeMethodHandleError::invalid_request(
            "url must not include credentials",
        ));
    }

    Ok(url)
}

fn external_url_approval(
    value: &str,
) -> Result<RustBridgeApprovalRequest, BridgeMethodHandleError> {
    let url = validated_external_url(value)?;

    Ok(RustBridgeApprovalRequest {
        body: RustBridgeApprovalBody::OpenExternalUrl {
            url: url.to_string(),
        },
    })
}

#[async_trait]
impl BridgeMethod for EnvironmentOpenExternalUrl {
    fn name(&self) -> &'static str {
        "environment.openExternalUrl"
    }

    fn capability(&self) -> BridgeMethodCapability {
        BridgeMethodCapability::user(UserBridgeCapability::EnvironmentOpenExternalUrl)
    }

    fn approval_request(
        &self,
        _ctx: BridgeContext<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeApprovalRequestResult {
        let params: EnvironmentOpenExternalUrlParams = parse_required_params(self, request)?;
        Ok(Some(external_url_approval(&params.url)?))
    }

    async fn handle(
        &self,
        _ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeHandleResult {
        let params: EnvironmentOpenExternalUrlParams = parse_required_params(self, request)?;
        let url = validated_external_url(&params.url)?;

        tools
            .app_handle
            .opener()
            .open_url(url.as_str(), None::<&str>)
            .map_err(|err| {
                BridgeMethodHandleError::internal_error(format!("{} failed: {err}", self.name()))
            })?;

        Ok(Box::new(EnvironmentOpenExternalUrlResult { opened: true }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_absolute_http_and_https_urls() {
        assert!(validated_external_url("https://example.com/docs?source=sage#apps").is_ok());
        assert!(validated_external_url("http://localhost:3000/help").is_ok());
    }

    #[test]
    fn rejects_non_web_schemes_and_relative_urls() {
        for value in [
            "javascript:alert(1)",
            "file:///tmp/example",
            "mailto:user@example.com",
            "/relative/path",
        ] {
            assert!(
                validated_external_url(value).is_err(),
                "unexpectedly accepted {value}"
            );
        }
    }

    #[test]
    fn rejects_embedded_credentials_and_oversized_urls() {
        assert!(validated_external_url("https://user:password@example.com").is_err());

        let oversized = format!(
            "https://example.com/{}",
            "a".repeat(MAX_EXTERNAL_URL_LENGTH)
        );
        assert!(validated_external_url(&oversized).is_err());
    }

    #[test]
    fn approval_contains_the_normalized_url_used_for_opening() {
        let approval = external_url_approval("HTTPS://Example.COM:443/docs").unwrap();

        let RustBridgeApprovalBody::OpenExternalUrl { url } = approval.body else {
            panic!("unexpected approval body");
        };

        assert_eq!(url, "https://example.com/docs");
    }
}
