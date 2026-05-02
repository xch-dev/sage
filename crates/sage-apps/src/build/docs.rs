use std::fmt::Write;
use std::{fs, path::PathBuf};

use crate::capabilities::list::{BridgeCapability, SystemBridgeCapability, UserBridgeCapability};
use crate::bridge::methods::BridgeMethodCapability;
use crate::bridge::registry::{BridgeRegistry, BridgeRegistryKind};
use crate::capabilities::{get_system_capability_definition, get_user_capability_definition};

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

pub fn user_capabilities_markdown() -> String {
    let mut out = String::from("# User bridge capabilities\n\n");

    for capability in UserBridgeCapability::ALL {
        let definition = get_user_capability_definition(*capability);

        writeln!(out, "## `{}`\n", definition.capability().key()).unwrap();
        writeln!(out, "**{}**\n", definition.label()).unwrap();
        writeln!(out, "{}\n", definition.description()).unwrap();

        out.push_str("| Flag | Value |\n");
        out.push_str("|---|---|\n");

        writeln!(
            out,
            "| Requestable by app | {} |",
            bool_cell(definition.flags().requestable_by_app())
        )
        .unwrap();

        writeln!(
            out,
            "| User grantable | {} |",
            bool_cell(definition.flags().user_grantable())
        )
        .unwrap();

        writeln!(
            out,
            "| Shared with app | {} |",
            bool_cell(definition.flags().shared_with_app())
        )
        .unwrap();

        writeln!(
            out,
            "| Externally observable | {} |",
            bool_cell(definition.flags().externally_observable())
        )
        .unwrap();

        writeln!(
            out,
            "| Accesses sensitive secret | {} |\n",
            bool_cell(definition.flags().accesses_sensitive_secret())
        )
        .unwrap();
    }

    out
}

pub fn system_capabilities_markdown() -> String {
    let mut out = String::from("# System bridge capabilities\n\n");

    for capability in SystemBridgeCapability::ALL {
        let definition = get_system_capability_definition(*capability);

        writeln!(out, "## `{}`\n", definition.capability().key()).unwrap();
        writeln!(out, "**{}**\n", definition.label()).unwrap();
        writeln!(out, "{}\n", definition.description()).unwrap();

        out.push_str("| Flag | Value |\n");
        out.push_str("|---|---|\n");

        writeln!(
            out,
            "| Requestable by app | {} |",
            bool_cell(definition.flags().requestable_by_app())
        )
        .unwrap();

        writeln!(
            out,
            "| User grantable | {} |",
            bool_cell(definition.flags().user_grantable())
        )
        .unwrap();

        writeln!(
            out,
            "| Shared with app | {} |",
            bool_cell(definition.flags().shared_with_app())
        )
        .unwrap();

        writeln!(
            out,
            "| Externally observable | {} |",
            bool_cell(definition.flags().externally_observable())
        )
        .unwrap();

        writeln!(
            out,
            "| Accesses sensitive secret | {} |\n",
            bool_cell(definition.flags().accesses_sensitive_secret())
        )
        .unwrap();
    }

    out
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

        out.push_str("| Field | Value |\n");
        out.push_str("|---|---|\n");

        writeln!(
            out,
            "| Capability | {} |",
            method_capability_cell(method.capability())
        )
        .unwrap();

        out.push('\n');
    }

    out
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
