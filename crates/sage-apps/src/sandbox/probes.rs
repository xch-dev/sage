mod isolation;
mod network;
mod persistence;
mod poll;

// How long a probe waits for its test webviews to report. The release waits 2-5 s,
// which a loaded machine (particularly Linux with per-context WebKitGTK processes)
// misses while the wallet is starting, and a missed report counts as a failed
// capability for the rest of the session. Waiting longer changes only when a probe
// gives up, never what it must observe to pass.
//
// 30 s since 2026-10-30: the network probe app now gives the ALLOWED fetch 12 s (was 5 s,
// example.com from a cold WebView2 behind a slow resolver) plus 5 s for the blocked one,
// and both must report inside this window.
pub(super) const PROBE_TIMEOUT_MS: i64 = 30_000;

pub(super) use isolation::*;
pub(super) use network::*;
pub(super) use persistence::*;
