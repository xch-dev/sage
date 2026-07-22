use std::fmt::Write;
use std::{fs, path::PathBuf};

use crate::{
    BridgeCapability, BridgeMethodCapability, BridgeRegistry, BridgeRegistryKind, CapabilityFlags,
    SystemBridgeCapability, UserBridgeCapability, get_system_capability_definition,
    get_user_capability_definition,
};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("failed to resolve workspace root")
        .to_path_buf()
}

fn write_if_changed(path: PathBuf, content: String) -> anyhow::Result<()> {
    if fs::read_to_string(&path).ok().as_deref() == Some(content.as_str()) {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, content)?;
    Ok(())
}

fn bool_cell(value: bool) -> &'static str {
    if value { "`true`" } else { "`false`" }
}

fn bridge_capability_key(capability: BridgeCapability) -> &'static str {
    match capability {
        BridgeCapability::User(capability) => capability.key(),
        BridgeCapability::System(capability) => capability.key(),
    }
}

fn method_capability_cell(capability: BridgeMethodCapability) -> String {
    match capability {
        BridgeMethodCapability::Ungated => "`ungated`".to_string(),
        BridgeMethodCapability::Required(cap) => {
            format!("`{}`", bridge_capability_key(cap))
        }
    }
}

fn push_markdown_table(out: &mut String, headers: (&str, &str), rows: &[(&str, String)]) {
    let first_width = rows
        .iter()
        .map(|(first, _)| first.len())
        .chain([headers.0.len()])
        .max()
        .unwrap_or(3)
        .max(3);
    let second_width = rows
        .iter()
        .map(|(_, second)| second.len())
        .chain([headers.1.len()])
        .max()
        .unwrap_or(3)
        .max(3);

    writeln!(
        out,
        "| {:first_width$} | {:second_width$} |",
        headers.0, headers.1
    )
    .unwrap();
    writeln!(
        out,
        "| {} | {} |",
        "-".repeat(first_width),
        "-".repeat(second_width)
    )
    .unwrap();

    for (first, second) in rows {
        writeln!(out, "| {first:first_width$} | {second:second_width$} |").unwrap();
    }
}

fn finish_markdown(mut out: String) -> String {
    let content_len = out.trim_end_matches('\n').len();
    out.truncate(content_len);
    out.push('\n');
    out
}

fn capability_flag_rows(flags: CapabilityFlags) -> [(&'static str, String); 5] {
    [
        (
            "Requestable by app",
            bool_cell(flags.requestable_by_app()).to_string(),
        ),
        (
            "User grantable",
            bool_cell(flags.user_grantable()).to_string(),
        ),
        (
            "Shared with app",
            bool_cell(flags.shared_with_app()).to_string(),
        ),
        (
            "Externally observable",
            bool_cell(flags.externally_observable()).to_string(),
        ),
        (
            "Accesses sensitive secret",
            bool_cell(flags.accesses_sensitive_secret()).to_string(),
        ),
    ]
}

pub fn user_capabilities_markdown() -> String {
    let mut out = String::from("# User bridge capabilities\n\n");

    for capability in UserBridgeCapability::ALL {
        let definition = get_user_capability_definition(*capability);

        writeln!(out, "## `{}`\n", definition.capability().key()).unwrap();
        writeln!(out, "**{}**\n", definition.label()).unwrap();
        writeln!(out, "{}\n", definition.description()).unwrap();

        push_markdown_table(
            &mut out,
            ("Flag", "Value"),
            &capability_flag_rows(definition.flags()),
        );
        out.push('\n');
    }

    finish_markdown(out)
}

pub fn system_capabilities_markdown() -> String {
    let mut out = String::from("# System bridge capabilities\n\n");

    for capability in SystemBridgeCapability::ALL {
        let definition = get_system_capability_definition(*capability);

        writeln!(out, "## `{}`\n", definition.capability().key()).unwrap();
        writeln!(out, "**{}**\n", definition.label()).unwrap();
        writeln!(out, "{}\n", definition.description()).unwrap();

        push_markdown_table(
            &mut out,
            ("Flag", "Value"),
            &capability_flag_rows(definition.flags()),
        );
        out.push('\n');
    }

    finish_markdown(out)
}

pub(crate) fn bridge_methods_markdown(kind: BridgeRegistryKind) -> String {
    let title = match kind {
        BridgeRegistryKind::User => "User bridge methods",
        BridgeRegistryKind::System => "System bridge methods",
    };

    let registry = BridgeRegistry::new(kind);
    let mut methods = registry.iter().collect::<Vec<_>>();
    methods.sort_by_key(|(name, _)| *name);

    let mut out = format!("# {title}\n\n");

    for (name, method) in methods {
        writeln!(out, "## `{name}`\n").unwrap();

        push_markdown_table(
            &mut out,
            ("Field", "Value"),
            &[("Capability", method_capability_cell(method.capability()))],
        );
        out.push('\n');
    }

    finish_markdown(out)
}

pub fn generate_docs() -> anyhow::Result<()> {
    let docs = workspace_root().join("docs").join("generated");

    write_if_changed(
        docs.join("user-bridge-capabilities.md"),
        user_capabilities_markdown(),
    )?;

    write_if_changed(
        docs.join("system-bridge-capabilities.md"),
        system_capabilities_markdown(),
    )?;

    write_if_changed(
        docs.join("user-bridge-methods.md"),
        bridge_methods_markdown(BridgeRegistryKind::User),
    )?;

    write_if_changed(
        docs.join("system-bridge-methods.md"),
        bridge_methods_markdown(BridgeRegistryKind::System),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{finish_markdown, push_markdown_table};

    #[test]
    fn markdown_table_matches_prettier_alignment() {
        let mut output = String::new();
        push_markdown_table(
            &mut output,
            ("Field", "Value"),
            &[("Capability", "`app_update.apply`".to_string())],
        );

        assert_eq!(
            output,
            "| Field      | Value              |\n| ---------- | ------------------ |\n| Capability | `app_update.apply` |\n"
        );
    }

    #[test]
    fn markdown_has_exactly_one_trailing_newline() {
        assert_eq!(finish_markdown("content\n\n".to_string()), "content\n");
    }
}
