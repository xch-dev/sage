use serde_json::{Value, json};

use crate::{SageAgent, UserBridgeCapability};

pub(crate) struct AgentTool {
    pub name: &'static str,
    pub bridge_method: &'static str,
    pub capability: UserBridgeCapability,
    pub description: &'static str,
    pub input_schema: fn() -> Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AgentCallError {
    pub code: &'static str,
    pub message: String,
}

fn no_params() -> Value {
    json!({ "type": "object", "properties": {}, "additionalProperties": false })
}

fn asset_coin_type() -> Value {
    json!({ "type": "string", "enum": ["cat", "did", "nft"] })
}

fn asset_balance_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "type": asset_coin_type(),
            "assetId": { "type": "string", "description": "Required when type is cat." }
        },
        "additionalProperties": false
    })
}

fn asset_coins_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "type": asset_coin_type(),
            "assetId": { "type": "string", "description": "Asset id (hex)." },
            "includedLocked": { "type": "boolean" },
            "offset": { "type": "integer", "minimum": 0 },
            "limit": { "type": "integer", "minimum": 0 }
        },
        "additionalProperties": false
    })
}

fn coins_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "asset_id": { "type": ["string", "null"], "description": "Omit or null for XCH." },
            "offset": { "type": "integer", "minimum": 0 },
            "limit": { "type": "integer", "minimum": 0 },
            "sort_mode": {
                "type": "string",
                "enum": [
                    "coin_id", "amount", "created_height", "spent_height",
                    "clawback_timestamp", "clawback_version"
                ]
            },
            "filter_mode": {
                "type": "string",
                "enum": ["all", "selectable", "owned", "spent", "clawback"]
            },
            "ascending": { "type": "boolean" }
        },
        "required": ["offset", "limit"]
    })
}

fn coins_by_ids_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "coin_ids": { "type": "array", "items": { "type": "string" } }
        },
        "required": ["coin_ids"]
    })
}

fn spendable_coin_count_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "asset_id": { "type": ["string", "null"], "description": "Omit or null for XCH." }
        }
    })
}

fn transactions_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "offset": { "type": "integer", "minimum": 0 },
            "limit": { "type": "integer", "minimum": 0 },
            "ascending": { "type": "boolean" },
            "find_value": { "type": ["string", "null"] }
        },
        "required": ["offset", "limit", "ascending"]
    })
}

fn transaction_schema() -> Value {
    json!({
        "type": "object",
        "properties": { "height": { "type": "integer", "minimum": 0 } },
        "required": ["height"]
    })
}

fn check_address_schema() -> Value {
    json!({
        "type": "object",
        "properties": { "address": { "type": "string" } },
        "required": ["address"]
    })
}

static TOOLS: &[AgentTool] = &[
    AgentTool {
        name: "get_wallet",
        bridge_method: "wallet.getWallet",
        capability: UserBridgeCapability::WalletGetWallet,
        description: "Get the currently selected wallet's name and fingerprint.",
        input_schema: no_params,
    },
    AgentTool {
        name: "get_sync_status",
        bridge_method: "wallet.getSyncStatus",
        capability: UserBridgeCapability::WalletGetSyncStatus,
        description: "Get the wallet's sync status and balance summary.",
        input_schema: no_params,
    },
    AgentTool {
        name: "get_version",
        bridge_method: "wallet.getVersion",
        capability: UserBridgeCapability::WalletGetVersion,
        description: "Get the Sage version.",
        input_schema: no_params,
    },
    AgentTool {
        name: "get_asset_balance",
        bridge_method: "wallet.getAssetBalance",
        capability: UserBridgeCapability::WalletGetAssetBalance,
        description: "Get the confirmed and spendable balance of an asset (XCH by default).",
        input_schema: asset_balance_schema,
    },
    AgentTool {
        name: "get_asset_coins",
        bridge_method: "wallet.getAssetCoins",
        capability: UserBridgeCapability::WalletGetAssetCoins,
        description: "List spendable coins of an asset (XCH by default).",
        input_schema: asset_coins_schema,
    },
    AgentTool {
        name: "get_coins",
        bridge_method: "wallet.getCoins",
        capability: UserBridgeCapability::WalletGetCoins,
        description: "List wallet coins with filtering, sorting and pagination.",
        input_schema: coins_schema,
    },
    AgentTool {
        name: "get_coins_by_ids",
        bridge_method: "wallet.getCoinsByIds",
        capability: UserBridgeCapability::WalletGetCoinsByIds,
        description: "Get wallet coins by their coin ids.",
        input_schema: coins_by_ids_schema,
    },
    AgentTool {
        name: "get_spendable_coin_count",
        bridge_method: "wallet.getSpendableCoinCount",
        capability: UserBridgeCapability::WalletGetSpendableCoinCount,
        description: "Count spendable coins of an asset (XCH by default).",
        input_schema: spendable_coin_count_schema,
    },
    AgentTool {
        name: "get_transactions",
        bridge_method: "wallet.getTransactions",
        capability: UserBridgeCapability::WalletGetTransactions,
        description: "List wallet transactions with pagination.",
        input_schema: transactions_schema,
    },
    AgentTool {
        name: "get_transaction",
        bridge_method: "wallet.getTransaction",
        capability: UserBridgeCapability::WalletGetTransaction,
        description: "Get a transaction by block height.",
        input_schema: transaction_schema,
    },
    AgentTool {
        name: "get_pending_transactions",
        bridge_method: "wallet.getPendingTransactions",
        capability: UserBridgeCapability::WalletGetPendingTransactions,
        description: "List transactions that have been submitted but not yet confirmed.",
        input_schema: no_params,
    },
    AgentTool {
        name: "get_xch_usd_price",
        bridge_method: "wallet.getXchUsdPrice",
        capability: UserBridgeCapability::WalletGetXchUsdPrice,
        description: "Get the current XCH price in USD.",
        input_schema: no_params,
    },
    AgentTool {
        name: "check_address",
        bridge_method: "wallet.checkAddress",
        capability: UserBridgeCapability::WalletCheckAddress,
        description: "Check whether an address is valid for this wallet's network.",
        input_schema: check_address_schema,
    },
];

#[cfg(test)]
pub(crate) fn agent_tools() -> &'static [AgentTool] {
    TOOLS
}

pub(crate) fn find_agent_tool(name: &str) -> Option<&'static AgentTool> {
    TOOLS.iter().find(|tool| tool.name == name)
}

pub(crate) fn find_agent_tool_by_bridge_method(method: &str) -> Option<&'static AgentTool> {
    TOOLS.iter().find(|tool| tool.bridge_method == method)
}

pub(crate) fn agent_grantable_capabilities() -> Vec<UserBridgeCapability> {
    let mut capabilities: Vec<_> = TOOLS.iter().map(|tool| tool.capability).collect();
    capabilities.sort();
    capabilities.dedup();
    capabilities
}

pub(crate) fn tools_for_agent(agent: &SageAgent) -> Vec<&'static AgentTool> {
    TOOLS
        .iter()
        .filter(|tool| agent.capabilities.contains(&tool.capability))
        .collect()
}

pub(crate) fn authorize_agent_call(
    agent: &SageAgent,
    tool: &AgentTool,
    current_fingerprint: Option<u32>,
) -> Result<(), AgentCallError> {
    if !agent.capabilities.contains(&tool.capability) {
        return Err(AgentCallError {
            code: "permission_denied",
            message: format!("agent is not granted access to {}", tool.name),
        });
    }

    let Some(fingerprint) = current_fingerprint else {
        return Err(AgentCallError {
            code: "no_wallet",
            message: "no wallet is currently logged in".to_string(),
        });
    };

    if !agent.is_wallet_in_scope(fingerprint) {
        return Err(AgentCallError {
            code: "wallet_not_in_scope",
            message: "the current wallet is outside this agent's wallet scope".to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::{
        BridgeMethodCapability, BridgeRegistry, BridgeRegistryKind, SageAppWalletScope,
        get_user_capability_definition,
    };

    use UserBridgeCapability::*;

    fn agent(caps: &[UserBridgeCapability], scope: SageAppWalletScope) -> SageAgent {
        SageAgent {
            id: "a".to_string(),
            name: "agent".to_string(),
            capabilities: caps.iter().copied().collect::<BTreeSet<_>>(),
            wallet_scope: scope,
            created_at_ms: 0,
            updated_at_ms: 0,
            last_used_at_ms: None,
        }
    }

    fn tool(name: &str) -> &'static AgentTool {
        find_agent_tool(name).expect("tool exists")
    }

    #[test]
    fn catalog_matches_bridge_registry() {
        let registry = BridgeRegistry::new(BridgeRegistryKind::User);
        for tool in agent_tools() {
            let method = registry
                .get(tool.bridge_method)
                .unwrap_or_else(|| panic!("{} missing from registry", tool.bridge_method));
            assert_eq!(
                method.capability(),
                BridgeMethodCapability::user(tool.capability),
                "{}",
                tool.name
            );
        }
    }

    #[test]
    fn catalog_has_no_secret_or_signing_capabilities() {
        let registry = BridgeRegistry::new(BridgeRegistryKind::User);
        for tool in agent_tools() {
            assert!(
                !get_user_capability_definition(tool.capability)
                    .flags()
                    .accesses_sensitive_secret(),
                "{}",
                tool.name
            );
            assert!(
                !registry.get(tool.bridge_method).unwrap().requires_signing(),
                "{}",
                tool.name
            );
        }
    }

    #[test]
    fn catalog_is_read_only() {
        let forbidden = [
            WalletSendXch,
            WalletSendXchAutoSubmit,
            WalletSignCoinSpends,
            WalletSignMessage,
            WalletSendTransaction,
            WalletGetSecretKey,
        ];
        for tool in agent_tools() {
            assert!(!forbidden.contains(&tool.capability), "{}", tool.name);
        }
        for cap in agent_grantable_capabilities() {
            assert!(!forbidden.contains(&cap));
        }
    }

    #[test]
    fn grantable_capabilities_are_sorted_and_unique() {
        let caps = agent_grantable_capabilities();
        let mut expected = caps.clone();
        expected.sort();
        expected.dedup();
        assert_eq!(caps, expected);
        assert_eq!(caps.len(), agent_tools().len());
    }

    #[test]
    fn schemas_are_objects() {
        for tool in agent_tools() {
            assert_eq!((tool.input_schema)()["type"], "object", "{}", tool.name);
        }
    }

    #[test]
    fn tools_filtered_by_grants() {
        let agent = agent(
            &[WalletGetWallet, WalletGetSyncStatus],
            SageAppWalletScope::AllWallets,
        );
        let mut names: Vec<_> = tools_for_agent(&agent).iter().map(|t| t.name).collect();
        names.sort_unstable();
        assert_eq!(names, vec!["get_sync_status", "get_wallet"]);
    }

    #[test]
    fn call_refused_without_grant() {
        let agent = agent(&[WalletGetSyncStatus], SageAppWalletScope::AllWallets);
        let err = authorize_agent_call(&agent, tool("get_wallet"), Some(1)).unwrap_err();
        assert_eq!(err.code, "permission_denied");
    }

    #[test]
    fn call_refused_without_current_wallet() {
        let agent = agent(&[WalletGetWallet], SageAppWalletScope::AllWallets);
        let err = authorize_agent_call(&agent, tool("get_wallet"), None).unwrap_err();
        assert_eq!(err.code, "no_wallet");
    }

    #[test]
    fn call_refused_when_current_wallet_out_of_scope() {
        let agent = agent(
            &[WalletGetWallet],
            SageAppWalletScope::SelectedWallets {
                fingerprints: vec![7],
            },
        );
        let err = authorize_agent_call(&agent, tool("get_wallet"), Some(8)).unwrap_err();
        assert_eq!(err.code, "wallet_not_in_scope");
    }

    #[test]
    fn call_allowed_in_scope_with_grant() {
        let agent = agent(
            &[WalletGetWallet],
            SageAppWalletScope::SelectedWallets {
                fingerprints: vec![7],
            },
        );
        assert_eq!(
            authorize_agent_call(&agent, tool("get_wallet"), Some(7)),
            Ok(())
        );
    }
}
