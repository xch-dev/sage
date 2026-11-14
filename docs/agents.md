# AI agents (MCP)

Sage can let AI agents, such as Claude Code, read your wallet over the [Model Context Protocol](https://modelcontextprotocol.io). Stage 1 is read-only: agents can look at balances, coins and transactions, but cannot sign or send anything. Agents are available on desktop only.

Each agent gets its own token. A token only grants the capabilities and wallets you chose for that agent, and you can edit, rotate or revoke it at any time.

## Enable the server

1. Open **Settings → Advanced → Agents**.
2. Turn on the agent server. It listens on `127.0.0.1` only, over plain HTTP, on port 9258 by default. The port can be changed (1024 to 65535).
3. The server is off by default and stays off until you enable it.

## Add an agent

1. Click **Add agent** and give it a name.
2. Choose its wallet scope: the current wallet (the default) or all wallets.
3. Choose its capabilities. All are checked by default.
4. Click **Create**. The token is shown once. Copy it now, because Sage stores only a hash and cannot show it again.

Sage also shows two ready-to-use snippets, filled in with the real port and token.

Claude Code:

```sh
claude mcp add --transport http sage http://127.0.0.1:9258/mcp --header "Authorization: Bearer <token>"
```

Other MCP clients (JSON config):

```json
{
  "mcpServers": {
    "sage": {
      "type": "http",
      "url": "http://127.0.0.1:9258/mcp",
      "headers": { "Authorization": "Bearer <token>" }
    }
  }
}
```

## Tools (stage 1)

| Tool                       | What it does                      |
| -------------------------- | --------------------------------- |
| `get_wallet`               | Current wallet information        |
| `get_sync_status`          | Sync status of the wallet         |
| `get_version`              | Sage version                      |
| `get_asset_balance`        | Balance of an asset               |
| `get_asset_coins`          | Coins of an asset                 |
| `get_coins`                | Coins in the wallet               |
| `get_coins_by_ids`         | Look up coins by id               |
| `get_spendable_coin_count` | Number of spendable coins         |
| `get_transactions`         | Transaction history               |
| `get_transaction`          | A single transaction              |
| `get_pending_transactions` | Pending transactions              |
| `get_xch_usd_price`        | Current XCH price in USD          |
| `check_address`            | Check whether an address is valid |

An agent only sees the tools matching the capabilities you granted. Secret keys are never available to agents.

Every tool acts on the wallet currently logged in to Sage. If no wallet is logged in, every call fails with `no_wallet`, including `get_version` and `get_xch_usd_price`.

## Edit, rotate and revoke

- **Edit** changes an agent's name, wallet scope and capabilities. Changes apply on the agent's very next request.
- **Rotate token** (in the edit dialog) issues a new token and immediately invalidates the old one. Update your client with the new token.
- **Revoke** deletes the agent after a confirmation. Its token stops working immediately, even for connected clients.

If the current wallet is outside an agent's scope, its calls fail until you switch back.

## Security

A token only grants its capabilities. But an agent that can run shell commands as your OS user can do anything you can. That includes reading Sage's RPC certificate while the RPC server is enabled, which gives full wallet access and bypasses agent approvals.

- Run such agents sandboxed, or as a different OS user.
- Keep the RPC server off while using agents. The Agents settings show a warning whenever the RPC server is running.
- Treat tokens like passwords. Revoke any agent you no longer use.
