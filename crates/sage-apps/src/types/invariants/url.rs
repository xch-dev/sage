use anyhow::Result as AnyResult;
use url::Url;

pub fn normalize_app_url(mut url: Url) -> AnyResult<Url> {
    validate_app_url(&url)?;

    url.set_query(None);
    url.set_fragment(None);

    if !url.path().ends_with('/') {
        let path = url.path().trim_end_matches('/');
        url.set_path(&format!("{path}/"));
    }

    Ok(url)
}

pub fn validate_app_url(url: &Url) -> AnyResult<()> {
    match url.scheme() {
        "https" => {}
        "http" if is_localhost(url) => {}
        scheme => {
            anyhow::bail!(
                "unsupported app URL scheme '{scheme}', only https is allowed except http://localhost"
            );
        }
    }

    if url.host_str().is_none() {
        anyhow::bail!("app URL must include a host");
    }

    Ok(())
}

fn is_localhost(url: &Url) -> bool {
    matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "::1"))
}
