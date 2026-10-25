//! Rewrites NFT URIs that name content-addressed data (IPFS and Arweave) into
//! the HTTPS URLs that can actually be fetched, including fallbacks for when
//! the host baked into the NFT stops serving the content.
//!
//! This follows the approach the reference wallet (chia-blockchain-gui) took:
//!
//! * `ipfs://<CID>[/path]` URIs can't be requested directly, so they're
//!   translated to HTTPS gateway URLs. The redundant `ipfs://ipfs/<CID>` form
//!   produced by some minting tools is tolerated.
//! * Plain gateway links (`https://host/ipfs/<CID>/...` or
//!   `https://<CID>.ipfs.host/...`) name content by CID exactly like an
//!   `ipfs://` URI does, so when their own host fails (as happened to every
//!   `nftstorage.link` NFT) the same bytes can be fetched from another gateway.
//!
//! The same is done for Arweave (`ar://<txid>[/path]` and gateway links).
//!
//! Gateways don't need to be trusted for integrity, since every file is
//! checked against the hash committed to on-chain before it's marked as
//! matching.

use reqwest::Url;

/// HTTPS gateways that `ipfs://` content is fetched through, in order of
/// preference. ipfs.io is last because it's rate limited for automated
/// clients and is expected to become more restrictive over time.
pub const IPFS_GATEWAYS: [&str; 3] = [
    "https://ipfs.filebase.io/ipfs/",
    "https://gateway.pinata.cloud/ipfs/",
    "https://ipfs.io/ipfs/",
];

/// HTTPS gateways that `ar://` content is fetched through, in order of
/// preference.
pub const ARWEAVE_GATEWAYS: [&str; 2] = ["https://arweave.net/", "https://permagate.io/"];

/// Hosts (and their subdomains) that serve Arweave transactions at
/// `https://host/<txid>[/path]`.
const ARWEAVE_GATEWAY_HOSTS: [&str; 4] =
    ["arweave.net", "permagate.io", "ar-io.net", "arweave.dev"];

/// NFT URIs come from the chain with no length limit, so anything longer than
/// this is refused up front rather than worked on.
const MAX_URL_LENGTH: usize = 2084;

/// A CIDv1 in base32 or base36 is at least this long, which is what tells a
/// subdomain gateway (`<CID>.ipfs.host`) apart from a path gateway whose
/// hostname merely contains `ipfs` (`gw.ipfs.example.com`).
const MIN_SUBDOMAIN_CID_LENGTH: usize = 46;

/// Arweave transaction ids are 32 bytes encoded as unpadded base64url.
const ARWEAVE_TXID_LENGTH: usize = 43;

/// Returns the URLs to try for an NFT URI, in order. The first entry is the
/// URI itself if it can be requested as-is, followed by any gateway fallbacks.
/// This always returns at least one URL.
pub fn uri_candidates(uri: &str) -> Vec<String> {
    if let Some(path) = ipfs_path(uri) {
        return gateway_urls(&IPFS_GATEWAYS, &path, None);
    }

    if let Some(path) = arweave_path(uri) {
        return gateway_urls(&ARWEAVE_GATEWAYS, &path, None);
    }

    if let Some((path, host)) = ipfs_path_from_gateway_url(uri) {
        let mut urls = vec![uri.to_string()];
        urls.extend(gateway_urls(&IPFS_GATEWAYS, &path, Some(&host)));
        return urls;
    }

    if let Some((path, host)) = arweave_path_from_gateway_url(uri) {
        let mut urls = vec![uri.to_string()];
        urls.extend(gateway_urls(&ARWEAVE_GATEWAYS, &path, Some(&host)));
        return urls;
    }

    vec![uri.to_string()]
}

/// Appends a content path to each gateway base, skipping the gateway that
/// already failed (if any). A URL that would resolve anywhere other than under
/// the gateway's base is dropped.
fn gateway_urls(gateways: &[&str], path: &str, skip_host: Option<&str>) -> Vec<String> {
    gateways
        .iter()
        .filter(|base| skip_host.is_none_or(|host| gateway_host(base).as_deref() != Some(host)))
        .filter_map(|base| {
            let url = Url::parse(&format!("{base}{path}")).ok()?;
            url.as_str().starts_with(base).then(|| url.to_string())
        })
        .collect()
}

/// The gateway operator serving a URL: its hostname, less the `<CID>.ipfs.`
/// label of a subdomain gateway and the sandbox label of an Arweave gateway.
fn gateway_host(url: &str) -> Option<String> {
    let host = Url::parse(url).ok()?.host_str()?.to_ascii_lowercase();

    if let Some((label, rest)) = host.split_once(".ipfs.")
        && is_subdomain_cid(label)
    {
        return Some(rest.to_string());
    }

    if let Some(gateway) = arweave_gateway_host(&host) {
        return Some(gateway.to_string());
    }

    Some(host)
}

/// Returns the `<CID>[/path][?query]` part of an `ipfs://` URI.
fn ipfs_path(uri: &str) -> Option<String> {
    let rest = strip_scheme(uri, "ipfs://")?;
    let rest = strip_prefix_ignore_case(rest, "ipfs/").unwrap_or(rest);
    content_path(rest, is_cid)
}

/// Returns the `<txid>[/path][?query]` part of an `ar://` URI.
fn arweave_path(uri: &str) -> Option<String> {
    let rest = strip_scheme(uri, "ar://")?;
    content_path(rest, is_arweave_txid)
}

/// Returns the `<CID>[/path][?query]` part of an HTTPS IPFS gateway link, as
/// well as the gateway operator serving it.
fn ipfs_path_from_gateway_url(uri: &str) -> Option<(String, String)> {
    let url = parse_http_url(uri)?;
    let host = url.host_str()?.to_ascii_lowercase();

    // The subdomain form is tried first, so that a subdomain gateway serving a
    // directory whose own path starts with `/ipfs/` keeps its CID.
    let (path, gateway) = if let Some((label, gateway)) = host.split_once(".ipfs.")
        && is_subdomain_cid(label)
    {
        (format!("{label}{}", url.path()), gateway.to_string())
    } else {
        let path = strip_prefix_ignore_case(url.path(), "/ipfs/")?;
        (path.to_string(), host)
    };

    let path = with_query(&path, url.query());
    Some((content_path(&path, is_cid)?, gateway))
}

/// Returns the `<txid>[/path][?query]` part of an HTTPS Arweave gateway link,
/// as well as the gateway operator serving it.
fn arweave_path_from_gateway_url(uri: &str) -> Option<(String, String)> {
    let url = parse_http_url(uri)?;
    let host = url.host_str()?.to_ascii_lowercase();
    let gateway = arweave_gateway_host(&host)?;
    let path = with_query(url.path().trim_start_matches('/'), url.query());
    Some((content_path(&path, is_arweave_txid)?, gateway.to_string()))
}

/// The known Arweave gateway a hostname belongs to, including the sandbox
/// subdomains (`<base32>.arweave.net`) that gateways redirect to.
fn arweave_gateway_host(host: &str) -> Option<&'static str> {
    ARWEAVE_GATEWAY_HOSTS.into_iter().find(|gateway| {
        host == *gateway
            || host
                .strip_suffix(gateway)
                .is_some_and(|prefix| prefix.ends_with('.'))
    })
}

fn parse_http_url(uri: &str) -> Option<Url> {
    if uri.len() > MAX_URL_LENGTH {
        return None;
    }

    let url = Url::parse(uri).ok()?;

    if !matches!(url.scheme(), "http" | "https") {
        return None;
    }

    Some(url)
}

fn with_query(path: &str, query: Option<&str>) -> String {
    match query {
        Some(query) => format!("{path}?{query}"),
        None => path.to_string(),
    }
}

fn strip_scheme<'a>(uri: &'a str, scheme: &str) -> Option<&'a str> {
    if uri.len() > MAX_URL_LENGTH {
        return None;
    }

    strip_prefix_ignore_case(uri.trim(), scheme)
}

fn strip_prefix_ignore_case<'a>(value: &'a str, prefix: &str) -> Option<&'a str> {
    let head = value.get(..prefix.len())?;
    head.eq_ignore_ascii_case(prefix)
        .then(|| &value[prefix.len()..])
}

/// Validates `<id>[/path][?query][#fragment]` and returns it without the
/// fragment (which is never sent) or trailing slashes. The value comes from
/// data the minter wrote and ends up appended to a gateway base, so an id that
/// isn't valid or any path segment that would resolve somewhere other than
/// under the base disqualifies the whole thing.
fn content_path(value: &str, is_valid_id: fn(&str) -> bool) -> Option<String> {
    let value = value.split('#').next().unwrap_or_default();
    let (path, query) = match value.split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (value, None),
    };
    let path = path.trim_end_matches('/');

    let mut segments = path.split('/');
    let id = segments.next()?;

    if !is_valid_id(id) || !segments.all(is_safe_segment) {
        return None;
    }

    Some(with_query(path, query))
}

/// A CID is base58btc (v0) or base32/base36 (v1): letters and digits only.
/// CIDv0 is case-sensitive, so it's never case-normalized.
fn is_cid(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

fn is_subdomain_cid(value: &str) -> bool {
    value.len() >= MIN_SUBDOMAIN_CID_LENGTH && is_cid(value)
}

fn is_arweave_txid(value: &str) -> bool {
    value.len() == ARWEAVE_TXID_LENGTH
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

/// Whether a path segment stays under the gateway base once appended to it:
/// not empty, not `.` or `..` (or their percent-encoded spellings), and free
/// of backslashes (which URL parsing turns into slashes), whitespace, and
/// control characters.
fn is_safe_segment(segment: &str) -> bool {
    if segment.is_empty() || segment.contains(['\\']) {
        return false;
    }

    if segment
        .chars()
        .any(|char| char.is_whitespace() || char.is_control())
    {
        return false;
    }

    let decoded = segment.to_ascii_lowercase().replace("%2e", ".");
    decoded != "." && decoded != ".."
}

#[cfg(test)]
mod tests {
    use super::*;

    const CID_V1: &str = "bafybeicsfvwqp25vgqoc5azurhtdklb7imhlxsjb4lyvkt2mqgx2jibmi4";
    const CID_V0: &str = "QmT78zSuBmuS4z925WZfrqQ1qHaJ56DQaTfyMUF7F8ff5o";
    const TXID: &str = "bNbA3TEQVL60xlgCcqdz4ZPHFZ711cZ3hmkpGttDt_U";

    fn ipfs_gateways(path: &str) -> Vec<String> {
        IPFS_GATEWAYS
            .iter()
            .map(|base| format!("{base}{path}"))
            .collect()
    }

    #[test]
    fn test_plain_https_is_unchanged() {
        let uri = "https://example.com/nft.png";
        assert_eq!(uri_candidates(uri), vec![uri.to_string()]);
    }

    #[test]
    fn test_ipfs_scheme() {
        assert_eq!(
            uri_candidates(&format!("ipfs://{CID_V1}")),
            ipfs_gateways(CID_V1)
        );
        assert_eq!(
            uri_candidates(&format!("ipfs://{CID_V1}/1.png")),
            ipfs_gateways(&format!("{CID_V1}/1.png"))
        );
    }

    #[test]
    fn test_ipfs_scheme_variants() {
        // Redundant `ipfs/` prefix, uppercase scheme, and trailing slash.
        assert_eq!(
            uri_candidates(&format!("IPFS://ipfs/{CID_V1}/")),
            ipfs_gateways(CID_V1)
        );

        // CIDv0 is case-sensitive.
        assert_eq!(
            uri_candidates(&format!("ipfs://{CID_V0}")),
            ipfs_gateways(CID_V0)
        );

        // Queries are kept and fragments are dropped.
        assert_eq!(
            uri_candidates(&format!("ipfs://{CID_V1}/a.json?x=1#frag")),
            ipfs_gateways(&format!("{CID_V1}/a.json?x=1"))
        );
    }

    #[test]
    fn test_ipfs_scheme_rejects_escapes() {
        for uri in [
            format!("ipfs://{CID_V1}/../evil"),
            format!("ipfs://{CID_V1}/%2e%2E/evil"),
            format!("ipfs://{CID_V1}/a\\b"),
            format!("ipfs://{CID_V1}//a"),
            "ipfs://not-a-cid/a".to_string(),
            "ipfs://".to_string(),
        ] {
            assert_eq!(uri_candidates(&uri), vec![uri.clone()], "{uri}");
        }
    }

    #[test]
    fn test_path_gateway_link() {
        let uri = format!("https://nftstorage.link/ipfs/{CID_V1}/1.png");
        let mut expected = vec![uri.clone()];
        expected.extend(ipfs_gateways(&format!("{CID_V1}/1.png")));
        assert_eq!(uri_candidates(&uri), expected);
    }

    #[test]
    fn test_subdomain_gateway_link() {
        let uri = format!("https://{CID_V1}.ipfs.nftstorage.link/1.png?a=b");
        let mut expected = vec![uri.clone()];
        expected.extend(ipfs_gateways(&format!("{CID_V1}/1.png?a=b")));
        assert_eq!(uri_candidates(&uri), expected);
    }

    #[test]
    fn test_gateway_link_skips_own_gateway() {
        let uri = format!("https://ipfs.io/ipfs/{CID_V1}");
        let mut expected = vec![uri.clone()];
        expected.extend(
            ipfs_gateways(CID_V1)
                .into_iter()
                .filter(|url| !url.starts_with("https://ipfs.io/")),
        );
        assert_eq!(uri_candidates(&uri), expected);
    }

    #[test]
    fn test_path_gateway_with_ipfs_in_hostname() {
        let uri = format!("https://gw.ipfs.example.com/ipfs/{CID_V1}");
        let mut expected = vec![uri.clone()];
        expected.extend(ipfs_gateways(CID_V1));
        assert_eq!(uri_candidates(&uri), expected);
    }

    #[test]
    fn test_non_cid_gateway_path_is_unchanged() {
        let uri = "https://example.com/ipfs/not-a-cid/1.png";
        assert_eq!(uri_candidates(uri), vec![uri.to_string()]);
    }

    #[test]
    fn test_arweave_scheme() {
        assert_eq!(
            uri_candidates(&format!("ar://{TXID}")),
            vec![
                format!("https://arweave.net/{TXID}"),
                format!("https://permagate.io/{TXID}"),
            ]
        );
        assert_eq!(
            uri_candidates(&format!("ar://{TXID}/0.png")),
            vec![
                format!("https://arweave.net/{TXID}/0.png"),
                format!("https://permagate.io/{TXID}/0.png"),
            ]
        );
        assert_eq!(uri_candidates("ar://short"), vec!["ar://short".to_string()]);
    }

    #[test]
    fn test_arweave_gateway_link() {
        let uri = format!("https://arweave.net/{TXID}");
        assert_eq!(
            uri_candidates(&uri),
            vec![uri.clone(), format!("https://permagate.io/{TXID}")]
        );

        let uri = format!(
            "https://ntlmbxjrcbkl5ngglabhfj3t4gj4ofm66xk4m54gneurvw2dw72q.arweave.net/{TXID}"
        );
        assert_eq!(
            uri_candidates(&uri),
            vec![uri.clone(), format!("https://permagate.io/{TXID}")]
        );

        let uri = format!("https://ar-io.net/{TXID}");
        assert_eq!(
            uri_candidates(&uri),
            vec![
                uri.clone(),
                format!("https://arweave.net/{TXID}"),
                format!("https://permagate.io/{TXID}"),
            ]
        );
    }

    #[test]
    fn test_lookalike_arweave_host_is_unchanged() {
        let uri = format!("https://notarweave.net/{TXID}");
        assert_eq!(uri_candidates(&uri), vec![uri.clone()]);
    }
}
