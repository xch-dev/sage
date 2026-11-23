use tauri::{AppHandle, State};

use super::super::runtime::{start_test_app, stop_test_apps, unique_run_id};
use super::super::store::SandboxAppResult;
use super::super::types::SandboxNetworkProbeResult;
use super::poll::poll_network;
use crate::{AppsHostState, BUILTIN_NETWORK_ALLOW_A_ID, BUILTIN_NETWORK_ALLOW_B_ID};

pub(in crate::sandbox) async fn run_network_test(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
) -> Result<(bool, Option<String>), String> {
    let run_id = unique_run_id("sandbox-network");
    let app_ids = [BUILTIN_NETWORK_ALLOW_A_ID, BUILTIN_NETWORK_ALLOW_B_ID];

    stop_test_apps(app, apps_state, &app_ids).await?;

    let probe_result = async {
        start_test_app(
            app,
            apps_state,
            BUILTIN_NETWORK_ALLOW_A_ID,
            &[("runId", run_id.clone())],
        )
        .await?;
        start_test_app(
            app,
            apps_state,
            BUILTIN_NETWORK_ALLOW_B_ID,
            &[("runId", run_id.clone())],
        )
        .await?;

        poll_network(apps_state, &run_id, 2, super::PROBE_TIMEOUT_MS).await
    }
    .await;

    let stop_result = stop_test_apps(app, apps_state, &app_ids).await;
    let results = probe_result?;
    stop_result?;

    network_verdict(&results)
}

/// The network capability's verdict from both probe apps' reports.
///
///  a URL outside the allowlist answering is the one thing this probe
/// exists to catch. It is looked for in EVERY report first and is a finished verdict
/// (`Ok(false)`) that the runner never retries. A probe app that crashed, or an ALLOWED URL
/// out of reach (example.com slow or down, a captive portal, the machine offline at login),
/// says nothing about the sandbox: the probe could not finish (`Err`), so the runner may try
/// again; on the last attempt the capability still fails, as upstream's does.
pub(crate) fn network_verdict(
    results: &[SandboxAppResult<SandboxNetworkProbeResult>],
) -> Result<(bool, Option<String>), String> {
    if let Some(result) = results.iter().find(|r| r.data.blocked_ok) {
        return Ok((
            false,
            Some(format!(
                "{} was able to reach blocked URL {}.",
                result.app_id, result.data.blocked_url
            )),
        ));
    }

    for result in results {
        if let Some(error) = &result.data.error {
            return Err(format!(
                "{} network probe could not finish: {error}",
                result.app_id
            ));
        }

        if !result.data.allowed_ok {
            return Err(format!(
                "{} could not reach allowed URL {}.",
                result.app_id, result.data.allowed_url
            ));
        }
    }

    Ok((
        true,
        Some("Network allowlist probes succeeded for allowed URLs and failed for blocked URLs in both flipped configurations.".into()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(
        app: &str,
        allowed_ok: bool,
        blocked_ok: bool,
        error: Option<&str>,
    ) -> SandboxAppResult<SandboxNetworkProbeResult> {
        SandboxAppResult {
            app_id: app.into(),
            data: SandboxNetworkProbeResult {
                run_id: "r".into(),
                allowed_url: "https://example.com/".into(),
                blocked_url: "https://example.org/".into(),
                allowed_ok,
                blocked_ok,
                error: error.map(str::to_string),
            },
        }
    }

    #[test]
    fn both_probes_right_passes() {
        let got = network_verdict(&[
            report("a", true, false, None),
            report("b", true, false, None),
        ]);
        assert!(matches!(got, Ok((true, _))));
    }

    #[test]
    fn a_reached_blocked_url_is_a_finished_failure_even_behind_an_unreachable_allowed_url() {
        // a could not reach its allowed URL (would be retryable), b reached a BLOCKED one:
        // the violation must win, or a retry could wash it out.
        let got = network_verdict(&[
            report("a", false, false, None),
            report("b", true, true, None),
        ]);
        assert!(
            matches!(got, Ok((false, Some(ref m))) if m.contains("blocked")),
            "{got:?}"
        );
    }

    #[test]
    fn unreachable_allowed_url_or_a_crash_is_unfinished() {
        assert!(
            network_verdict(&[
                report("a", false, false, None),
                report("b", true, false, None)
            ])
            .is_err()
        );
        assert!(
            network_verdict(&[
                report("a", true, false, Some("boom")),
                report("b", true, false, None)
            ])
            .is_err()
        );
    }
}
