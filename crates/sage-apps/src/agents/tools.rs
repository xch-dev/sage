use sage_api::{
    CheckAddress, GetCoins, GetCoinsByIds, GetSpendableCoinCount, GetTransaction, GetTransactions,
};
use serde_json::{Value, json};

use crate::{
    BridgeMethod, BridgeMethodHandleError, BridgeRegistry, SageAgent, UserBridgeCapability,
    WalletGetAssetBalanceParams, WalletGetAssetCoinsParams, schema_of,
};

pub(crate) struct AgentTool {
    pub name: &'static str,
    pub bridge_method: &'static str,
    pub capability: UserBridgeCapability,
    pub description: &'static str,
    pub input_schema: fn() -> Value,
}

fn no_params() -> Value {
    json!({ "type": "object", "properties": {}, "additionalProperties": false })
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
        input_schema: schema_of::<WalletGetAssetBalanceParams>,
    },
    AgentTool {
        name: "get_asset_coins",
        bridge_method: "wallet.getAssetCoins",
        capability: UserBridgeCapability::WalletGetAssetCoins,
        description: "List spendable coins of an asset (XCH by default).",
        input_schema: schema_of::<WalletGetAssetCoinsParams>,
    },
    AgentTool {
        name: "get_coins",
        bridge_method: "wallet.getCoins",
        capability: UserBridgeCapability::WalletGetCoins,
        description: "List wallet coins with filtering, sorting and pagination.",
        input_schema: schema_of::<GetCoins>,
    },
    AgentTool {
        name: "get_coins_by_ids",
        bridge_method: "wallet.getCoinsByIds",
        capability: UserBridgeCapability::WalletGetCoinsByIds,
        description: "Get wallet coins by their coin ids.",
        input_schema: schema_of::<GetCoinsByIds>,
    },
    AgentTool {
        name: "get_spendable_coin_count",
        bridge_method: "wallet.getSpendableCoinCount",
        capability: UserBridgeCapability::WalletGetSpendableCoinCount,
        description: "Count spendable coins of an asset (XCH by default).",
        input_schema: schema_of::<GetSpendableCoinCount>,
    },
    AgentTool {
        name: "get_transactions",
        bridge_method: "wallet.getTransactions",
        capability: UserBridgeCapability::WalletGetTransactions,
        description: "List wallet transactions with pagination.",
        input_schema: schema_of::<GetTransactions>,
    },
    AgentTool {
        name: "get_transaction",
        bridge_method: "wallet.getTransaction",
        capability: UserBridgeCapability::WalletGetTransaction,
        description: "Get a transaction by block height.",
        input_schema: schema_of::<GetTransaction>,
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
        input_schema: schema_of::<CheckAddress>,
    },
];

#[cfg(test)]
pub(crate) fn agent_tools() -> &'static [AgentTool] {
    TOOLS
}

pub(crate) fn find_agent_tool(name: &str) -> Option<&'static AgentTool> {
    TOOLS.iter().find(|tool| tool.name == name)
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

/// Checks that an agent may call a tool on the current wallet, and finds its bridge method.
pub(crate) fn authorize_agent_call<'a>(
    registry: &'a BridgeRegistry,
    agent: &SageAgent,
    tool: &AgentTool,
    current_fingerprint: Option<u32>,
) -> Result<&'a dyn BridgeMethod, BridgeMethodHandleError> {
    if !agent.capabilities.contains(&tool.capability) {
        return Err(BridgeMethodHandleError::new(
            "permission_denied",
            format!("agent is not granted access to {}", tool.name),
        ));
    }

    let Some(method) = registry.get(tool.bridge_method) else {
        return Err(BridgeMethodHandleError::new(
            "method_not_found",
            format!("Unknown bridge method: {}", tool.bridge_method),
        ));
    };

    let Some(fingerprint) = current_fingerprint else {
        return Err(BridgeMethodHandleError::new(
            "no_wallet",
            "no wallet is currently logged in",
        ));
    };

    if !agent.is_wallet_in_scope(fingerprint) {
        return Err(BridgeMethodHandleError::new(
            "wallet_not_in_scope",
            "the current wallet is outside this agent's wallet scope",
        ));
    }

    Ok(method)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::{
        BridgeMethodCapability, BridgeRegistryKind, SageAppWalletScope,
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

    fn registry() -> BridgeRegistry {
        BridgeRegistry::new(BridgeRegistryKind::User)
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

    fn contains_ref(value: &Value) -> bool {
        match value {
            Value::Object(map) => map.contains_key("$ref") || map.values().any(contains_ref),
            Value::Array(items) => items.iter().any(contains_ref),
            _ => false,
        }
    }

    fn property_names(schema: &Value) -> BTreeSet<&str> {
        schema["properties"]
            .as_object()
            .expect("schema has properties")
            .keys()
            .map(String::as_str)
            .collect()
    }

    #[test]
    fn schemas_are_self_contained() {
        for tool in agent_tools() {
            assert!(!contains_ref(&(tool.input_schema)()), "{}", tool.name);
        }
    }

    #[test]
    fn rpc_typed_schema_inlines_enums() {
        let schema = (tool("get_coins").input_schema)();

        assert_eq!(
            property_names(&schema),
            BTreeSet::from([
                "asset_id",
                "offset",
                "limit",
                "sort_mode",
                "filter_mode",
                "ascending"
            ])
        );
        assert_eq!(schema["required"], json!(["offset", "limit"]));
        assert_eq!(schema["properties"]["sort_mode"]["type"], "string");
        assert_eq!(schema["properties"]["sort_mode"]["enum"][0], "coin_id");
    }

    #[test]
    fn bridge_typed_schema_follows_serde_names() {
        let schema = (tool("get_asset_coins").input_schema)();

        assert_eq!(
            property_names(&schema),
            BTreeSet::from(["type", "assetId", "includedLocked", "offset", "limit"])
        );
        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(
            schema["properties"]["type"]["oneOf"][1]["enum"],
            json!(["cat", "did", "nft"])
        );
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
        let err = authorize_agent_call(&registry(), &agent, tool("get_wallet"), Some(1))
            .err()
            .expect("refused");
        assert_eq!(err.code, "permission_denied");
    }

    #[test]
    fn call_refused_without_current_wallet() {
        let agent = agent(&[WalletGetWallet], SageAppWalletScope::AllWallets);
        let err = authorize_agent_call(&registry(), &agent, tool("get_wallet"), None)
            .err()
            .expect("refused");
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
        let err = authorize_agent_call(&registry(), &agent, tool("get_wallet"), Some(8))
            .err()
            .expect("refused");
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
        let registry = registry();
        let result = authorize_agent_call(&registry, &agent, tool("get_wallet"), Some(7));
        assert_eq!(
            result.map(BridgeMethod::name).ok(),
            Some("wallet.getWallet")
        );
    }

    #[test]
    fn call_refused_for_a_method_missing_from_the_registry() {
        let tool = AgentTool {
            name: "missing",
            bridge_method: "wallet.missing",
            capability: WalletGetWallet,
            description: "",
            input_schema: no_params,
        };
        let agent = agent(&[WalletGetWallet], SageAppWalletScope::AllWallets);
        let err = authorize_agent_call(&registry(), &agent, &tool, Some(1))
            .err()
            .expect("refused");
        assert_eq!(err.code, "method_not_found");
    }
}
