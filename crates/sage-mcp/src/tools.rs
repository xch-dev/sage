use sage_api::{
    CheckAddress, GetCoins, GetCoinsByIds, GetSpendableCoinCount, GetToken, GetTransaction,
    GetTransactions,
};
use sage_auth::{Agent, Capability};
use serde_json::{Value, json};

use crate::schema_of;

#[derive(Debug, Clone, Copy)]
pub struct AgentTool {
    /// The sage-api endpoint name, which is also the capability's endpoint.
    pub name: &'static str,
    pub capability: Capability,
    pub description: &'static str,
    pub input_schema: fn() -> Value,
}

fn no_params() -> Value {
    json!({ "type": "object", "properties": {}, "additionalProperties": false })
}

static TOOLS: &[AgentTool] = &[
    AgentTool {
        name: "get_wallet",
        capability: Capability::WalletGetWallet,
        description: "Get the currently selected wallet's name and fingerprint.",
        input_schema: no_params,
    },
    AgentTool {
        name: "get_sync_status",
        capability: Capability::WalletGetSyncStatus,
        description: "Get the wallet's sync status and balance summary.",
        input_schema: no_params,
    },
    AgentTool {
        name: "get_version",
        capability: Capability::WalletGetVersion,
        description: "Get the Sage version.",
        input_schema: no_params,
    },
    AgentTool {
        name: "get_token",
        capability: Capability::WalletGetToken,
        description: "Get an asset's details and confirmed and spendable balance (XCH by default).",
        input_schema: schema_of::<GetToken>,
    },
    AgentTool {
        name: "get_coins",
        capability: Capability::WalletGetCoins,
        description: "List wallet coins with filtering, sorting and pagination.",
        input_schema: schema_of::<GetCoins>,
    },
    AgentTool {
        name: "get_coins_by_ids",
        capability: Capability::WalletGetCoinsByIds,
        description: "Get wallet coins by their coin ids.",
        input_schema: schema_of::<GetCoinsByIds>,
    },
    AgentTool {
        name: "get_spendable_coin_count",
        capability: Capability::WalletGetSpendableCoinCount,
        description: "Count spendable coins of an asset (XCH by default).",
        input_schema: schema_of::<GetSpendableCoinCount>,
    },
    AgentTool {
        name: "get_transactions",
        capability: Capability::WalletGetTransactions,
        description: "List wallet transactions with pagination.",
        input_schema: schema_of::<GetTransactions>,
    },
    AgentTool {
        name: "get_transaction",
        capability: Capability::WalletGetTransaction,
        description: "Get a transaction by block height.",
        input_schema: schema_of::<GetTransaction>,
    },
    AgentTool {
        name: "get_pending_transactions",
        capability: Capability::WalletGetPendingTransactions,
        description: "List transactions that have been submitted but not yet confirmed.",
        input_schema: no_params,
    },
    AgentTool {
        name: "get_xch_usd_price",
        capability: Capability::WalletGetXchUsdPrice,
        description: "Get the current XCH price in USD.",
        input_schema: no_params,
    },
    AgentTool {
        name: "check_address",
        capability: Capability::WalletCheckAddress,
        description: "Check whether an address belongs to this wallet.",
        input_schema: schema_of::<CheckAddress>,
    },
];

/// Every tool, in `Capability::ALL` order.
pub fn agent_tools() -> &'static [AgentTool] {
    TOOLS
}

pub fn find_agent_tool(name: &str) -> Option<&'static AgentTool> {
    TOOLS.iter().find(|tool| tool.name == name)
}

/// The tools whose capability the agent holds.
pub fn tools_for_agent(agent: &Agent) -> Vec<&'static AgentTool> {
    TOOLS
        .iter()
        .filter(|tool| agent.capabilities.contains(&tool.capability))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use sage_auth::WalletScope;

    use super::*;

    fn agent(caps: &[Capability]) -> Agent {
        Agent {
            id: "a".to_string(),
            name: "agent".to_string(),
            capabilities: caps.iter().copied().collect(),
            wallet_scope: WalletScope::AllWallets,
            created_at_ms: 0,
            updated_at_ms: 0,
            last_used_at_ms: None,
        }
    }

    fn tool(name: &str) -> &'static AgentTool {
        find_agent_tool(name).expect("tool exists")
    }

    #[test]
    fn catalog_has_one_tool_per_capability() {
        let names: Vec<_> = agent_tools().iter().map(|t| t.name).collect();
        let endpoints: Vec<_> = Capability::ALL
            .iter()
            .map(|c| c.endpoint().unwrap())
            .collect();
        assert_eq!(names, endpoints);
        for t in agent_tools() {
            assert_eq!(Some(t.name), t.capability.endpoint());
        }
    }

    #[test]
    fn get_wallet_takes_no_arguments() {
        let schema = (tool("get_wallet").input_schema)();
        assert_eq!(schema["properties"], json!({}));
        assert_eq!(schema["additionalProperties"], json!(false));
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
    fn tools_filtered_by_grants() {
        let agent = agent(&[Capability::WalletGetWallet, Capability::WalletGetSyncStatus]);
        let mut names: Vec<_> = tools_for_agent(&agent).iter().map(|t| t.name).collect();
        names.sort_unstable();
        assert_eq!(names, vec!["get_sync_status", "get_wallet"]);
    }
}
