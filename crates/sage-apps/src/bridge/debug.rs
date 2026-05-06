macro_rules! comms_debug {
    ($($arg:tt)*) => {
        if crate::bridge::sage_apps_comms_debug_enabled() {
            tracing::info!(target: "sage_apps_comms", $($arg)*);
        }
    };
}

pub(crate) use comms_debug;

pub(crate) fn sage_apps_comms_debug_enabled() -> bool {
    cfg!(debug_assertions)
        && std::env::var("SAGE_APPS_COMMS_DEBUG")
        .is_ok_and(|value| value == "1" || value.eq_ignore_ascii_case("true"))
}
