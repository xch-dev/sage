//! SSRF-guarded HTTP fetching for app installs and updates.
//!
//! App URLs are user-supplied, so downloads must not reach local or otherwise
//! non-public infrastructure. Each request resolves and vets its destination,
//! pins the connection to the vetted addresses, and validates redirect hops.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use anyhow::{Context, Result as AnyResult};
use url::{Host, Url};

const MAX_DOWNLOAD_REDIRECTS: usize = 5;

/// Fetches an app-related URL while validating every destination and redirect.
/// Status handling is deliberately left to the caller.
pub(crate) async fn get_with_ssrf_guard(url: &str) -> AnyResult<reqwest::Response> {
    let mut current = Url::parse(url).with_context(|| format!("invalid download URL {url}"))?;
    let initial_is_loopback = is_explicit_loopback_host(&current);

    for redirects in 0..=MAX_DOWNLOAD_REDIRECTS {
        // A public app must never gain permission to target loopback merely by
        // redirecting there or embedding a loopback asset URL. Local development
        // apps retain that permission for explicit loopback destinations.
        let allow_loopback = allows_loopback_target(initial_is_loopback, &current);
        let response = send_validated_get(&current, allow_loopback).await?;
        if !response.status().is_redirection() {
            return Ok(response);
        }

        if redirects == MAX_DOWNLOAD_REDIRECTS {
            anyhow::bail!("too many redirects while downloading {url}");
        }

        let location = response
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|value| value.to_str().ok())
            .with_context(|| format!("redirect from {current} is missing a location header"))?;

        current = current
            .join(location)
            .with_context(|| format!("invalid redirect location from {current}"))?;
    }

    unreachable!("redirect loop is bounded above")
}

async fn send_validated_get(url: &Url, allow_loopback: bool) -> AnyResult<reqwest::Response> {
    let pinned_addrs = validate_download_target(url, allow_loopback).await?;
    let mut builder = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        // A proxy receives the hostname rather than the vetted socket address,
        // so it would defeat DNS pinning and could reintroduce SSRF.
        .no_proxy();

    if let (Some(domain), Some(addrs)) = (url.domain(), pinned_addrs.as_deref()) {
        builder = builder.resolve_to_addrs(domain, addrs);
    }

    let client = builder.build().context("failed to build download client")?;
    client
        .get(url.clone())
        .send()
        .await
        .with_context(|| format!("failed to GET {url}"))
}

/// Validates a URL and, for domains, returns the resolved addresses to pin.
async fn validate_download_target(
    url: &Url,
    allow_loopback: bool,
) -> AnyResult<Option<Vec<SocketAddr>>> {
    match url.scheme() {
        "https" => {}
        "http" if allow_loopback => {}
        scheme => anyhow::bail!("refusing to download {url}: unsupported scheme '{scheme}'"),
    }

    match url
        .host()
        .with_context(|| format!("download URL {url} must include a host"))?
    {
        Host::Ipv4(ip) => {
            ensure_ip_allowed(IpAddr::V4(ip), allow_loopback, url)?;
            Ok(None)
        }
        Host::Ipv6(ip) => {
            ensure_ip_allowed(IpAddr::V6(ip), allow_loopback, url)?;
            Ok(None)
        }
        Host::Domain(domain) => {
            let port = url.port_or_known_default().unwrap_or(443);
            let addrs: Vec<SocketAddr> = tokio::net::lookup_host((domain, port))
                .await
                .with_context(|| format!("failed to resolve download host {domain}"))?
                .collect();

            if addrs.is_empty() {
                anyhow::bail!("download host {domain} did not resolve to any address");
            }

            for addr in &addrs {
                ensure_ip_allowed(addr.ip(), allow_loopback, url)?;
            }

            Ok(Some(addrs))
        }
    }
}

fn is_explicit_loopback_host(url: &Url) -> bool {
    match url.host() {
        Some(Host::Domain(domain)) => domain.eq_ignore_ascii_case("localhost"),
        Some(Host::Ipv4(ip)) => ip.is_loopback(),
        Some(Host::Ipv6(ip)) => ip.is_loopback(),
        None => false,
    }
}

fn allows_loopback_target(allow_loopback_source: bool, url: &Url) -> bool {
    allow_loopback_source && is_explicit_loopback_host(url)
}

fn ensure_ip_allowed(ip: IpAddr, allow_loopback: bool, url: &Url) -> AnyResult<()> {
    if is_forbidden_ip(ip, allow_loopback) {
        anyhow::bail!("refusing to download {url}: host resolves to disallowed address {ip}");
    }

    Ok(())
}

fn is_forbidden_ip(ip: IpAddr, allow_loopback: bool) -> bool {
    match ip {
        IpAddr::V4(ip) => is_forbidden_ipv4(ip, allow_loopback),
        IpAddr::V6(ip) => {
            if let Some(mapped) = ip.to_ipv4_mapped() {
                return is_forbidden_ipv4(mapped, allow_loopback);
            }

            is_forbidden_ipv6(ip, allow_loopback)
        }
    }
}

fn is_forbidden_ipv4(ip: Ipv4Addr, allow_loopback: bool) -> bool {
    if ip.is_loopback() {
        return !allow_loopback;
    }

    let [first, second, third, _] = ip.octets();
    ip.is_private()
        || ip.is_link_local()
        || ip.is_unspecified()
        || ip.is_broadcast()
        || ip.is_documentation()
        || ip.is_multicast()
        // "This network" (0.0.0.0/8), CGNAT (100.64.0.0/10), IETF protocol
        // assignments (192.0.0.0/24), deprecated 6to4 relay (192.88.99.0/24),
        // benchmark testing (198.18.0.0/15), and reserved space (240.0.0.0/4).
        || first == 0
        || (first == 100 && (second & 0xc0) == 64)
        || (first == 192 && second == 0 && third == 0)
        || (first == 192 && second == 88 && third == 99)
        || (first == 198 && matches!(second, 18 | 19))
        || first >= 240
}

fn is_forbidden_ipv6(ip: Ipv6Addr, allow_loopback: bool) -> bool {
    if ip.is_loopback() {
        return !allow_loopback;
    }

    let segments = ip.segments();
    let first = segments[0];

    ip.is_unspecified()
        || ip.is_multicast()
        // IPv4-compatible addresses (::/96) are not public IPv6 destinations.
        || segments[..6].iter().all(|segment| *segment == 0)
        // Unique-local (fc00::/7), link-local (fe80::/10), and deprecated
        // site-local (fec0::/10) addresses.
        || (first & 0xfe00) == 0xfc00
        || (first & 0xffc0) == 0xfe80
        || (first & 0xffc0) == 0xfec0
        // Discard-only (100::/64), Teredo (2001::/32), ORCHIDv2
        // (2001:20::/28), documentation (2001:db8::/32), and 6to4 (2002::/16).
        || (first == 0x0100
            && segments[1] == 0
            && segments[2] == 0
            && segments[3] == 0)
        || (first == 0x2001 && segments[1] == 0)
        || (first == 0x2001 && (segments[1] & 0xfff0) == 0x0020)
        || (first == 0x2001 && segments[1] == 0x0db8)
        || first == 0x2002
}

#[cfg(test)]
mod tests {
    use super::*;

    fn forbidden(host: &str) -> bool {
        is_forbidden_ip(host.parse().unwrap(), false)
    }

    #[test]
    fn rejects_non_public_addresses() {
        for host in [
            "0.1.2.3",
            "10.0.0.1",
            "100.64.0.1",
            "127.0.0.1",
            "169.254.169.254",
            "172.16.0.1",
            "192.0.0.1",
            "192.88.99.1",
            "192.168.1.1",
            "198.18.0.1",
            "203.0.113.1",
            "240.0.0.1",
            "::",
            "::1",
            "::10.0.0.1",
            "::ffff:10.0.0.1",
            "fc00::1",
            "fe80::1",
            "fec0::1",
            "100::1",
            "2001::1",
            "2001:20::1",
            "2001:db8::1",
            "2002:c0a8:1::",
        ] {
            assert!(forbidden(host), "expected {host} to be rejected");
        }
    }

    #[test]
    fn accepts_public_addresses() {
        for host in ["1.1.1.1", "8.8.8.8", "104.16.0.1", "2606:4700::1111"] {
            assert!(!forbidden(host), "expected {host} to be accepted");
        }
    }

    #[test]
    fn loopback_requires_an_explicit_loopback_url() {
        assert!(is_forbidden_ip("127.0.0.1".parse().unwrap(), false));
        assert!(!is_forbidden_ip("127.0.0.1".parse().unwrap(), true));
        assert!(is_forbidden_ip("::1".parse().unwrap(), false));
        assert!(!is_forbidden_ip("::1".parse().unwrap(), true));

        assert!(is_forbidden_ip("10.0.0.1".parse().unwrap(), true));
        assert!(is_forbidden_ip("169.254.169.254".parse().unwrap(), true));
    }

    #[test]
    fn identifies_explicit_loopback_hosts() {
        for url in [
            "http://localhost:4173/app/",
            "http://127.0.0.1:4173/app/",
            "http://[::1]:4173/app/",
        ] {
            assert!(is_explicit_loopback_host(&Url::parse(url).unwrap()));
        }

        for url in ["https://example.com/app/", "https://10.0.0.1/app/"] {
            assert!(!is_explicit_loopback_host(&Url::parse(url).unwrap()));
        }
    }

    #[tokio::test]
    async fn rejects_private_ip_literal_downloads() {
        let url = Url::parse("https://169.254.169.254/latest/meta-data/").unwrap();
        let err = validate_download_target(&url, false).await.unwrap_err();
        assert!(err.to_string().contains("disallowed address"));
    }

    #[tokio::test]
    async fn only_an_explicit_local_install_can_fetch_loopback() {
        let local_install = Url::parse("http://127.0.0.1:4173/app/").unwrap();
        assert!(allows_loopback_target(
            is_explicit_loopback_host(&local_install),
            &local_install
        ));
        validate_download_target(&local_install, true)
            .await
            .unwrap();

        let public_install = Url::parse("https://example.com/app/").unwrap();
        let redirect_target = Url::parse("https://127.0.0.1:4173/app/").unwrap();
        assert!(!allows_loopback_target(
            is_explicit_loopback_host(&public_install),
            &redirect_target
        ));
        let err = validate_download_target(&redirect_target, false)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("disallowed address"));
    }
}
