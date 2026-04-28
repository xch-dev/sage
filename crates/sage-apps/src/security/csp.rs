use std::collections::BTreeSet;

use crate::types::SageAppCommon;

fn csp_source_list(items: &[String]) -> String {
    items.join(" ")
}

pub fn build_app_csp(app: &SageAppCommon) -> String {
    let default_src = csp_source_list(&["'self'".to_string()]);
    let script_src = csp_source_list(&["'self'".to_string(), "'wasm-unsafe-eval'".to_string()]);
    let style_src = csp_source_list(&["'self'".to_string(), "'unsafe-inline'".to_string()]);
    let img_src = csp_source_list(&[
        "'self'".to_string(),
        "data:".to_string(),
        "blob:".to_string(),
    ]);
    let font_src = csp_source_list(&["'self'".to_string(), "data:".to_string()]);
    let media_src = csp_source_list(&[
        "'self'".to_string(),
        "data:".to_string(),
        "blob:".to_string(),
    ]);
    let object_src = csp_source_list(&["'none'".to_string()]);
    let frame_ancestors = csp_source_list(&["'self'".to_string()]);
    let base_uri = csp_source_list(&["'none'".to_string()]);
    let form_action = csp_source_list(&["'none'".to_string()]);
    let worker_src = csp_source_list(&["'self'".to_string()]);

    let mut connect_sources = BTreeSet::from(["'self'".to_string()]);

    for entry in app.granted_permissions().network().whitelist() {
        connect_sources.insert(entry.as_permission_string());
    }

    let connect_src = csp_source_list(&connect_sources.into_iter().collect::<Vec<_>>());

    format!(
        "default-src {default_src}; \
         script-src {script_src}; \
         style-src {style_src}; \
         img-src {img_src}; \
         font-src {font_src}; \
         media-src {media_src}; \
         object-src {object_src}; \
         base-uri {base_uri}; \
         form-action {form_action}; \
         frame-ancestors {frame_ancestors}; \
         connect-src {connect_src}; \
         worker-src {worker_src};"
    )
}
