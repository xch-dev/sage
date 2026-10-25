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

use percent_encoding::percent_decode;
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

/// The shortest CID: a CIDv0 is 46 base58 characters, and a CIDv1 in base32
/// or base36 is longer. Requiring this keeps ordinary words (such as the
/// `logo` in `https://example.com/ipfs/logo`) from being treated as CIDs, and
/// tells a subdomain gateway (`<CID>.ipfs.host`) apart from a path gateway
/// whose hostname merely contains `ipfs` (`gw.ipfs.example.com`).
const MIN_CID_LENGTH: usize = 46;

/// How many rounds of percent-decoding a path segment is checked through.
const MAX_DECODE_ROUNDS: usize = 4;

/// Arweave transaction ids are 32 bytes encoded as unpadded base64url.
const ARWEAVE_TXID_LENGTH: usize = 43;

/// Returns the URLs to try for an NFT URI, in order. This always returns at
/// least one URL.
///
/// `ipfs://` and `ar://` URIs can't be requested directly, so they're always
/// translated to gateway URLs. For HTTPS gateway links, the link itself comes
/// first, and other gateways are only added if `include_fallbacks` is set.
/// Fallbacks should only be used when the result will be checked against a
/// known hash, since otherwise they'd hide the fact that the URI is broken.
pub fn uri_candidates(uri: &str, include_fallbacks: bool) -> Vec<String> {
    if let Some((path, query)) = ipfs_path(uri) {
        return gateway_urls(&IPFS_GATEWAYS, &path, query, None);
    }

    if let Some((path, query)) = arweave_path(uri) {
        return gateway_urls(&ARWEAVE_GATEWAYS, &path, query, None);
    }

    let mut urls = vec![uri.to_string()];

    if !include_fallbacks {
        return urls;
    }

    if let Some((path, query, host)) = ipfs_path_from_gateway_url(uri) {
        urls.extend(gateway_urls(&IPFS_GATEWAYS, &path, query, Some(&host)));
    } else if let Some((path, query, host)) = arweave_path_from_gateway_url(uri) {
        urls.extend(gateway_urls(&ARWEAVE_GATEWAYS, &path, query, Some(host)));
    }

    urls
}

/// Appends a content path to each gateway base, skipping the gateway that
/// already failed (if any). A URL that would resolve anywhere other than under
/// the gateway's base is dropped.
fn gateway_urls(
    gateways: &[&str],
    path: &str,
    query: Option<&str>,
    skip_host: Option<&str>,
) -> Vec<String> {
    gateways
        .iter()
        .filter_map(|base| {
            let url = Url::parse(&format!("{base}{path}")).ok()?;

            if skip_host.is_some_and(|host| url.host_str() == Some(host))
                || !url.as_str().starts_with(base)
            {
                return None;
            }

            Some(match query {
                Some(query) => format!("{url}?{query}"),
                None => url.to_string(),
            })
        })
        .collect()
}

/// Returns the `<CID>[/path]` and query parts of an `ipfs://` URI.
fn ipfs_path(uri: &str) -> Option<(String, Option<&str>)> {
    let rest = strip_scheme(uri, "ipfs://")?;
    let rest = strip_prefix_ignore_case(rest, "ipfs/").unwrap_or(rest);
    let (path, query) = split_query(rest);
    Some((content_path(path, is_cid)?, query))
}

/// Returns the `<txid>[/path]` and query parts of an `ar://` URI.
fn arweave_path(uri: &str) -> Option<(String, Option<&str>)> {
    let (path, query) = split_query(strip_scheme(uri, "ar://")?);
    Some((content_path(path, is_arweave_txid)?, query))
}

/// Returns the `<CID>[/path]` and query parts of an HTTPS IPFS gateway link,
/// as well as the gateway operator serving it.
fn ipfs_path_from_gateway_url(uri: &str) -> Option<(String, Option<&str>, String)> {
    let url = parse_http_url(uri)?;
    let host = url.host_str()?.to_ascii_lowercase();

    // The subdomain form is tried first, so that a subdomain gateway serving a
    // directory whose own path starts with `/ipfs/` keeps its CID.
    let (path, gateway) = if let Some((label, gateway)) = host.split_once(".ipfs.")
        && is_cid(label)
    {
        (format!("{label}{}", url.path()), gateway.to_string())
    } else {
        let path = strip_prefix_ignore_case(url.path(), "/ipfs/")?;
        (path.to_string(), host.clone())
    };

    let path = content_path(&path, is_cid)?;
    Some((path, split_query(uri).1, gateway))
}

/// Returns the `<txid>[/path]` and query parts of an HTTPS Arweave gateway
/// link, as well as the gateway operator serving it.
fn arweave_path_from_gateway_url(uri: &str) -> Option<(String, Option<&str>, &'static str)> {
    let url = parse_http_url(uri)?;
    let host = url.host_str()?.to_ascii_lowercase();
    let gateway = arweave_gateway_host(&host)?;
    let path = content_path(url.path().trim_start_matches('/'), is_arweave_txid)?;
    Some((path, split_query(uri).1, gateway))
}

/// Splits a URI into the part before the query and the query itself, as they
/// were written so that the query is passed along unchanged. The fragment is
/// dropped, since it's never sent to the server.
fn split_query(uri: &str) -> (&str, Option<&str>) {
    let uri = uri.split('#').next().unwrap_or_default();

    match uri.split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (uri, None),
    }
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

/// Validates `<id>[/path]` and returns it without trailing slashes. The value
/// comes from data the minter wrote and ends up appended to a gateway base, so
/// an id that isn't valid or any path segment that would resolve somewhere
/// other than under the base disqualifies the whole thing.
fn content_path(path: &str, is_valid_id: fn(&str) -> bool) -> Option<String> {
    let path = path.trim_end_matches('/');

    let mut segments = path.split('/');
    let id = segments.next()?;

    if !is_valid_id(id) || !segments.all(is_safe_segment) {
        return None;
    }

    Some(path.to_string())
}

/// A CID is base58btc (v0) or base32/base36 (v1): letters and digits only.
/// CIDv0 is case-sensitive, so it's never case-normalized.
fn is_cid(value: &str) -> bool {
    value.len() >= MIN_CID_LENGTH && value.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

fn is_arweave_txid(value: &str) -> bool {
    value.len() == ARWEAVE_TXID_LENGTH
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

/// Whether a path segment stays under the gateway base once appended to it.
/// It must not be empty or contain control characters or backslashes (URL
/// parsing strips the former and turns the latter into slashes). Spaces are
/// fine, since they get percent-encoded. Once percent-decoded (repeatedly, in
/// case something along the way decodes more than once), it must not be `.`
/// or `..`, or contain a slash, backslash, or control character.
fn is_safe_segment(segment: &str) -> bool {
    if segment.is_empty()
        || segment
            .chars()
            .any(|char| char == '\\' || char.is_control())
    {
        return false;
    }

    let mut decoded = segment.as_bytes().to_vec();

    for _ in 0..MAX_DECODE_ROUNDS {
        if decoded
            .iter()
            .any(|&byte| byte == b'/' || byte == b'\\' || byte.is_ascii_control())
            || decoded == b"."
            || decoded == b".."
        {
            return false;
        }

        let next: Vec<u8> = percent_decode(&decoded).collect();

        if next == decoded {
            return true;
        }

        decoded = next;
    }

    // Anything still changing after this many rounds of decoding is suspicious.
    false
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
        assert_eq!(uri_candidates(uri, true), vec![uri.to_string()]);
    }

    #[test]
    fn test_ipfs_scheme() {
        assert_eq!(
            uri_candidates(&format!("ipfs://{CID_V1}"), true),
            ipfs_gateways(CID_V1)
        );
        assert_eq!(
            uri_candidates(&format!("ipfs://{CID_V1}/1.png"), true),
            ipfs_gateways(&format!("{CID_V1}/1.png"))
        );
    }

    #[test]
    fn test_ipfs_scheme_variants() {
        // Redundant `ipfs/` prefix, uppercase scheme, and trailing slash.
        assert_eq!(
            uri_candidates(&format!("IPFS://ipfs/{CID_V1}/"), true),
            ipfs_gateways(CID_V1)
        );

        // CIDv0 is case-sensitive.
        assert_eq!(
            uri_candidates(&format!("ipfs://{CID_V0}"), true),
            ipfs_gateways(CID_V0)
        );

        // Queries are kept and fragments are dropped.
        assert_eq!(
            uri_candidates(&format!("ipfs://{CID_V1}/a.json?x=1#frag"), true),
            ipfs_gateways(&format!("{CID_V1}/a.json?x=1"))
        );
    }

    #[test]
    fn test_ipfs_scheme_rejects_escapes() {
        for uri in [
            format!("ipfs://{CID_V1}/../evil"),
            format!("ipfs://{CID_V1}/%2e%2E/evil"),
            format!("ipfs://{CID_V1}/a\\b"),
            format!("ipfs://{CID_V1}/a\nb"),
            format!("ipfs://{CID_V1}//a"),
            "ipfs://not-a-cid/a".to_string(),
            "ipfs://".to_string(),
        ] {
            assert_eq!(uri_candidates(&uri, true), vec![uri.clone()], "{uri}");
        }
    }

    #[test]
    fn test_path_gateway_link() {
        let uri = format!("https://nftstorage.link/ipfs/{CID_V1}/1.png");
        let mut expected = vec![uri.clone()];
        expected.extend(ipfs_gateways(&format!("{CID_V1}/1.png")));
        assert_eq!(uri_candidates(&uri, true), expected);
    }

    #[test]
    fn test_subdomain_gateway_link() {
        let uri = format!("https://{CID_V1}.ipfs.nftstorage.link/1.png?a=b");
        let mut expected = vec![uri.clone()];
        expected.extend(ipfs_gateways(&format!("{CID_V1}/1.png?a=b")));
        assert_eq!(uri_candidates(&uri, true), expected);
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
        assert_eq!(uri_candidates(&uri, true), expected);
    }

    #[test]
    fn test_path_gateway_with_ipfs_in_hostname() {
        let uri = format!("https://gw.ipfs.example.com/ipfs/{CID_V1}");
        let mut expected = vec![uri.clone()];
        expected.extend(ipfs_gateways(CID_V1));
        assert_eq!(uri_candidates(&uri, true), expected);
    }

    #[test]
    fn test_non_cid_gateway_path_is_unchanged() {
        let uri = "https://example.com/ipfs/not-a-cid/1.png";
        assert_eq!(uri_candidates(uri, true), vec![uri.to_string()]);
    }

    #[test]
    fn test_arweave_scheme() {
        assert_eq!(
            uri_candidates(&format!("ar://{TXID}"), true),
            vec![
                format!("https://arweave.net/{TXID}"),
                format!("https://permagate.io/{TXID}"),
            ]
        );
        assert_eq!(
            uri_candidates(&format!("ar://{TXID}/0.png"), true),
            vec![
                format!("https://arweave.net/{TXID}/0.png"),
                format!("https://permagate.io/{TXID}/0.png"),
            ]
        );
        assert_eq!(
            uri_candidates("ar://short", true),
            vec!["ar://short".to_string()]
        );
    }

    #[test]
    fn test_arweave_gateway_link() {
        let uri = format!("https://arweave.net/{TXID}");
        assert_eq!(
            uri_candidates(&uri, true),
            vec![uri.clone(), format!("https://permagate.io/{TXID}")]
        );

        let uri = format!(
            "https://ntlmbxjrcbkl5ngglabhfj3t4gj4ofm66xk4m54gneurvw2dw72q.arweave.net/{TXID}"
        );
        assert_eq!(
            uri_candidates(&uri, true),
            vec![uri.clone(), format!("https://permagate.io/{TXID}")]
        );

        let uri = format!("https://ar-io.net/{TXID}");
        assert_eq!(
            uri_candidates(&uri, true),
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
        assert_eq!(uri_candidates(&uri, true), vec![uri.clone()]);
    }

    #[test]
    fn test_gateway_link_without_fallbacks() {
        let uri = format!("https://nftstorage.link/ipfs/{CID_V1}/1.png");
        assert_eq!(uri_candidates(&uri, false), vec![uri.clone()]);

        let uri = format!("https://arweave.net/{TXID}");
        assert_eq!(uri_candidates(&uri, false), vec![uri.clone()]);

        // Schemes that can't be requested directly are still translated.
        assert_eq!(
            uri_candidates(&format!("ipfs://{CID_V1}"), false),
            ipfs_gateways(CID_V1)
        );
    }

    #[test]
    fn test_short_paths_are_not_cids() {
        for uri in ["ipfs://ipfs", "https://example.com/ipfs/logo"] {
            assert_eq!(uri_candidates(uri, true), vec![uri.to_string()], "{uri}");
        }
    }

    #[test]
    fn test_encoded_separators_are_rejected() {
        for segment in [
            "..%2F..%2Fipns",
            "a%5Cb",
            "a%00b",
            "%2e%2E",
            "a%0Ab",
            "%252e%252e",
            "a%252Fb",
        ] {
            let uri = format!("ipfs://{CID_V1}/{segment}");
            assert_eq!(uri_candidates(&uri, true), vec![uri.clone()], "{uri}");
        }
    }

    #[test]
    fn test_spaces_are_allowed() {
        assert_eq!(
            uri_candidates(&format!("ipfs://{CID_V1}/my%20image.png"), true),
            ipfs_gateways(&format!("{CID_V1}/my%20image.png"))
        );
        assert_eq!(
            uri_candidates(&format!("ipfs://{CID_V1}/my image.png"), true),
            ipfs_gateways(&format!("{CID_V1}/my%20image.png"))
        );
    }
}
