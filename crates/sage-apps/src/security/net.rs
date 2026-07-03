//! SSRF-guarded HTTP fetching for app installs and updates.
//!
//! App URLs are user-supplied, so a download must never be allowed to reach
//! internal infrastructure (cloud metadata services, routers, LAN hosts, ...).
//! Every request hop resolves the target host up front, rejects
//! private/link-local/otherwise non-public addresses, and pins the connection
//! to the vetted addresses so DNS cannot be rebound between validation and
//! connect. Redirects are followed manually so each hop is re-validated.
//!
//! Loopback is only allowed when the URL host is explicitly loopback
//! (`localhost` / `127.0.0.1` / `::1`), which is the supported local dev flow.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use anyhow::{Context, Result as AnyResult};
use url::{Host, Url};

const MAX_DOWNLOAD_REDIRECTS: usize = 5;

/// Performs a GET request for an app-related URL, following up to
/// [`MAX_DOWNLOAD_REDIRECTS`] redirects while validating every hop against the
/// SSRF guard. Returns the final (non-redirect) response; status handling is
/// left to the caller.
pub(crate) async fn get_with_ssrf_guard(url: &str) -> AnyResult<reqwest::Response> {
    let mut current = Url::parse(url).with_context(|| format!("invalid download URL {url}"))?;
    let mut redirects = 0usize;

    loop {
        let response = send_validated_get(&current).await?;

        if !response.status().is_redirection() {
            return Ok(response);
        }

        redirects += 1;
        if redirects > MAX_DOWNLOAD_REDIRECTS {
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
}

async fn send_validated_get(url: &Url) -> AnyResult<reqwest::Response> {
    let pinned_addrs = validate_download_target(url).await?;

    let mut builder = reqwest::Client::builder().redirect(reqwest::redirect::Policy::none());

    if let (Some(domain), Some(addrs)) = (url.domain(), pinned_addrs.as_deref()) {
        // Pin the connection to the addresses we just vetted so DNS cannot be
        // re-resolved (rebound) between validation and connect.
        builder = builder.resolve_to_addrs(domain, addrs);
    }

    let client = builder.build().context("failed to build download client")?;

    client
        .get(url.clone())
        .send()
        .await
        .with_context(|| format!("failed to GET {url}"))
}

/// Validates the URL's scheme and target address. For domain hosts, returns
/// the resolved addresses so the caller can pin the connection to them.
async fn validate_download_target(url: &Url) -> AnyResult<Option<Vec<SocketAddr>>> {
    let allow_loopback = is_explicit_loopback_host(url);

    match url.scheme() {
        "https" => {}
        "http" if allow_loopback => {}
        scheme => anyhow::bail!("refusing to download {url}: unsupported scheme '{scheme}'"),
    }

    let host = url
        .host()
        .with_context(|| format!("download URL {url} must include a host"))?;

    match host {
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

            if ip.is_loopback() {
                return !allow_loopback;
            }

            let first_segment = ip.segments()[0];

            ip.is_unspecified()
                || ip.is_multicast()
                // Unique-local addresses (fc00::/7).
                || (first_segment & 0xfe00) == 0xfc00
                // Link-local addresses (fe80::/10).
                || (first_segment & 0xffc0) == 0xfe80
        }
    }
}

fn is_forbidden_ipv4(ip: Ipv4Addr, allow_loopback: bool) -> bool {
    if ip.is_loopback() {
        return !allow_loopback;
    }

    let octets = ip.octets();

    ip.is_private()
        || ip.is_link_local()
        || ip.is_unspecified()
        || ip.is_broadcast()
        || ip.is_documentation()
        || ip.is_multicast()
        // Carrier-grade NAT (100.64.0.0/10).
        || (octets[0] == 100 && (octets[1] & 0xc0) == 64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn forbidden(host: &str) -> bool {
        let ip: IpAddr = host.parse().unwrap();
        is_forbidden_ip(ip, false)
    }

    #[test]
    fn rejects_non_public_addresses() {
        for host in [
            "127.0.0.1",
            "10.0.0.1",
            "172.16.0.1",
            "192.168.1.1",
            "169.254.169.254",
            "100.64.0.1",
            "0.0.0.0",
            "255.255.255.255",
            "::1",
            "::",
            "fc00::1",
            "fd12:3456::1",
            "fe80::1",
            "::ffff:10.0.0.1",
            "::ffff:127.0.0.1",
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
    fn loopback_allowed_only_when_explicit() {
        assert!(is_forbidden_ip("127.0.0.1".parse().unwrap(), false));
        assert!(!is_forbidden_ip("127.0.0.1".parse().unwrap(), true));
        assert!(is_forbidden_ip("::1".parse().unwrap(), false));
        assert!(!is_forbidden_ip("::1".parse().unwrap(), true));

        // Explicit loopback must not unlock other internal ranges.
        assert!(is_forbidden_ip("10.0.0.1".parse().unwrap(), true));
        assert!(is_forbidden_ip("169.254.169.254".parse().unwrap(), true));
    }

    #[test]
    fn explicit_loopback_hosts_detected() {
        for url in [
            "http://localhost:4173/app/",
            "http://127.0.0.1:4173/app/",
            "http://[::1]:4173/app/",
        ] {
            assert!(
                is_explicit_loopback_host(&Url::parse(url).unwrap()),
                "expected {url} to count as explicit loopback"
            );
        }

        for url in ["https://example.com/app/", "https://10.0.0.1/app/"] {
            assert!(
                !is_explicit_loopback_host(&Url::parse(url).unwrap()),
                "expected {url} not to count as explicit loopback"
            );
        }
    }
}
