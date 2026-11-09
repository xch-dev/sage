use anyhow::Result as AnyResult;
use url::{Host, Url};

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
        "http" if is_loopback_url(url) => {}
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

pub fn is_loopback_url(url: &Url) -> bool {
    match url.host() {
        Some(Host::Domain(domain)) => domain.eq_ignore_ascii_case("localhost"),
        Some(Host::Ipv4(ip)) => ip == std::net::Ipv4Addr::LOCALHOST,
        Some(Host::Ipv6(ip)) => ip == std::net::Ipv6Addr::LOCALHOST,
        None => false,
    }
}
