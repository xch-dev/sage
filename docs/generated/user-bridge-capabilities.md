# User bridge capabilities

## `storage.persistent_webview`

**Persistent storage**

Allows the app to store data on this device between sessions.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `true` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `bridge.send`

**Bridge messaging**

Allows the app to send messages through the Sage bridge. (Only for sandbox tests)

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `app.get_capabilities`

**Read granted capabilities**

Allows the app to read the capabilities currently visible to it.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `app.get_info`

**Read app information**

Allows the app to read its Sage app identity and permission information.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `app.lifecycle.ready_to_stop`

**Acknowledge app shutdown**

Allows the app to acknowledge that it is ready to stop after a lifecycle request.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `app.lifecycle.set_before_stop_listener`

**Listen before app shutdown**

Allows the app to register a before-stop lifecycle listener.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `app.request_capability_grant`

**Request additional capability**

Allows the app to request a capability grant after installation.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `app.request_network_whitelist_grant`

**Request network access**

Allows the app to request access to an additional network target after installation.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `wallet.get_keys`

**List wallet keys**

Allows the app to list wallet keys configured in Sage.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `true` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `wallet.get_key`

**Read wallet key**

Allows the app to read public information about a wallet key.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `true` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `wallet.get_secret_key`

**Read wallet secret key**

Allows the app to read wallet secrets, including the mnemonic or private key when available.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `true` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `true` |

## `wallet.send_xch`

**Send XCH**

Allows the app to request XCH transactions from your wallet.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `true` |
| Shared with app | `true` |
| Externally observable | `true` |
| Accesses sensitive secret | `false` |

## `wallet.send_xch_auto_submit`

**Automatic XCH send**

Allows the app to submit XCH transactions without asking for per-transaction approval.

| Flag | Value |
|---|---|
| Requestable by app | `false` |
| User grantable | `false` |
| Shared with app | `false` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `wallet.get_sync_status`

**Read sync status**

Allows the app to read wallet sync status and current wallet balance summary.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `true` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `wallet.get_version`

**Read wallet version**

Allows the app to read the current Sage wallet version.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `true` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `wallet.check_address`

**Check address**

Allows the app to validate whether an address belongs to this wallet.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `true` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `wallet.get_derivations`

**Read derivations**

Allows the app to read wallet derivation records and addresses.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `true` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `wallet.get_spendable_coin_count`

**Read spendable coin count**

Allows the app to read the number of spendable coins in the wallet.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `true` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `wallet.get_coins_by_ids`

**Read coins by IDs**

Allows the app to read specific wallet coin records by coin ID.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `true` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `wallet.get_coins`

**Read coins**

Allows the app to list wallet coins.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `true` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `wallet.get_pending_transactions`

**Read pending transactions**

Allows the app to read pending wallet transactions.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `true` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `wallet.get_transaction`

**Read transaction**

Allows the app to read a wallet transaction by height.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `true` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `wallet.get_transactions`

**Read transactions**

Allows the app to list wallet transactions.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `true` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `environment.theme.get_current`

**Read current theme**

Allows the app to read Sage's current theme.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `environment.theme.css_vars`

**Use Sage theme CSS variables**

Allows Sage to inject current theme CSS variables into the app runtime.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `environment.theme.listen_changed`

**Observe theme changes**

Allows the app to receive events when Sage's theme changes.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

