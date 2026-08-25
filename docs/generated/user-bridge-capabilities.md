# User bridge capabilities

## `bridge.send`

**Bridge messaging**

Allows the app to send messages through the Sage bridge. (Only for sandbox tests)

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `false` |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |

## `app.get_info`

**Read app information**

Allows the app to read its Sage app identity and permission information.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `false` |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |

## `app.lifecycle.ready_to_stop`

**Acknowledge app shutdown**

Allows the app to acknowledge that it is ready to stop after a lifecycle request.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `false` |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |

## `app.lifecycle.set_before_stop_listener`

**Listen before app shutdown**

Allows the app to register a before-stop lifecycle listener.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `false` |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |

## `app.get_capabilities`

**Read granted capabilities**

Allows the app to read the capabilities currently visible to it.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `false` |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |

## `app.request_capability_grant`

**Request additional capability**

Allows the app to request a capability grant after installation.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `false` |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |

## `app.request_network_whitelist_grant`

**Request network access**

Allows the app to request access to an additional network target after installation.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `false` |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |

## `app.request_permission_grants`

**Request additional permissions**

Allows the app to request additional capabilities and network targets in one approval after installation.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `false` |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |

## `wallet.get_key`

**Read wallet key**

Allows the app to read public information about a wallet key.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `true`  |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |

## `wallet.get_secret_key`

**Read wallet secret key**

Allows the app to read wallet secrets, including the mnemonic or private key when available.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `true`  |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `true`  |

## `wallet.send_xch`

**Send XCH**

Allows the app to request XCH transactions from your wallet.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `true`  |
| Shared with app           | `true`  |
| Externally observable     | `true`  |
| Accesses sensitive secret | `false` |

## `wallet.send_xch_auto_submit`

**Automatic XCH send**

Allows the app to submit XCH transactions without asking for per-transaction approval.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `false` |
| User grantable            | `true`  |
| Shared with app           | `false` |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |

## `wallet.get_sync_status`

**Read sync status**

Allows the app to read wallet sync status and current wallet balance summary.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `true`  |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |

## `wallet.get_version`

**Read wallet version**

Allows the app to read the current Sage wallet version.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `true`  |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |

## `wallet.get_xch_usd_price`

**Read XCH/USD price**

Allows the app to read the current estimated XCH price in USD.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `false` |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |

## `wallet.check_address`

**Check address**

Allows the app to validate whether an address belongs to this wallet.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `true`  |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |

## `wallet.filter_unlocked_coins`

**Filter unlocked wallet coins**

Allows the app to check which supplied coin IDs are currently spendable.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `true`  |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |

## `wallet.get_asset_coins`

**Read wallet asset coins**

Allows the app to list spendable XCH, CAT, DID, or NFT coins.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `true`  |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |

## `wallet.get_asset_balance`

**Read wallet asset balance**

Allows the app to read confirmed and spendable balances for XCH, CAT, DID, or NFT assets.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `true`  |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |

## `wallet.sign_coin_spends`

**Sign wallet coin spends**

Allows the app to request signatures for custom coin spends after per-request approval.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `true`  |
| Shared with app           | `true`  |
| Externally observable     | `true`  |
| Accesses sensitive secret | `false` |

## `wallet.sign_message`

**Sign wallet messages**

Allows the app to request a message signature after per-request approval.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `true`  |
| Shared with app           | `true`  |
| Externally observable     | `true`  |
| Accesses sensitive secret | `false` |

## `wallet.send_transaction`

**Broadcast wallet transactions**

Allows the app to submit an already signed spend bundle to the Chia network.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `true`  |
| Shared with app           | `true`  |
| Externally observable     | `true`  |
| Accesses sensitive secret | `false` |

## `wallet.get_public_keys`

**Read wallet public keys**

Allows the app to read public keys derived by the active wallet.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `true`  |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |

## `wallet.get_derivations`

**Read derivations**

Allows the app to read wallet derivation records and addresses.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `true`  |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |

## `wallet.get_spendable_coin_count`

**Read spendable coin count**

Allows the app to read the number of spendable coins in the wallet.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `true`  |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |

## `wallet.get_coins_by_ids`

**Read coins by IDs**

Allows the app to read specific wallet coin records by coin ID.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `true`  |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |

## `wallet.get_coins`

**Read coins**

Allows the app to list wallet coins.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `true`  |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |

## `wallet.get_pending_transactions`

**Read pending transactions**

Allows the app to read pending wallet transactions.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `true`  |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |

## `wallet.get_transaction`

**Read transaction**

Allows the app to read a wallet transaction by height.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `true`  |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |

## `wallet.get_transactions`

**Read transactions**

Allows the app to list wallet transactions.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `true`  |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |

## `environment.theme.get_current`

**Read current theme**

Allows the app to read Sage's current theme.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `false` |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |

## `environment.theme.css_vars`

**Use Sage theme CSS variables**

Allows Sage to inject current theme CSS variables into the app runtime.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `false` |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |

## `environment.theme.listen_changed`

**Observe theme changes**

Allows the app to receive events when Sage's theme changes.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `false` |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |

## `environment.get_network`

**Read current network**

Allows the app to read Sage's currently active network information.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `false` |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |

## `environment.open_external_url`

**Open external links**

Allows the app to request opening an HTTP or HTTPS link in your default browser. Every link requires approval, and the destination can observe the request.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `false` |
| Shared with app           | `true`  |
| Externally observable     | `true`  |
| Accesses sensitive secret | `false` |

## `storage.persistent_webview`

**Persistent storage**

Allows the app to store data on this device between sessions.

| Flag                      | Value   |
| ------------------------- | ------- |
| Requestable by app        | `true`  |
| User grantable            | `true`  |
| Shared with app           | `true`  |
| Externally observable     | `false` |
| Accesses sensitive secret | `false` |
