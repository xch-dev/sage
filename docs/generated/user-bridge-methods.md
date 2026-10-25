# User bridge methods

## `app.getCapabilities`

| Field      | Value                  |
| ---------- | ---------------------- |
| Capability | `app.get_capabilities` |

## `app.getInfo`

| Field      | Value          |
| ---------- | -------------- |
| Capability | `app.get_info` |

## `app.lifecycle.readyToStop`

| Field      | Value                         |
| ---------- | ----------------------------- |
| Capability | `app.lifecycle.ready_to_stop` |

## `app.lifecycle.setBeforeStopListener`

| Field      | Value                                    |
| ---------- | ---------------------------------------- |
| Capability | `app.lifecycle.set_before_stop_listener` |

## `app.requestCapabilityGrant`

> **Deprecated:** Use `app.requestPermissionGrants` instead. It supports both single and batched capability and network permission requests.

| Field      | Value                          |
| ---------- | ------------------------------ |
| Capability | `app.request_capability_grant` |

## `app.requestNetworkWhitelistGrant`

> **Deprecated:** Use `app.requestPermissionGrants` instead. It supports both single and batched capability and network permission requests.

| Field      | Value                                 |
| ---------- | ------------------------------------- |
| Capability | `app.request_network_whitelist_grant` |

## `app.requestPermissionGrants`

| Field      | Value                           |
| ---------- | ------------------------------- |
| Capability | `app.request_permission_grants` |

## `bridge.ping`

| Field      | Value     |
| ---------- | --------- |
| Capability | `ungated` |

## `bridge.send`

| Field      | Value         |
| ---------- | ------------- |
| Capability | `bridge.send` |

## `environment.getNetwork`

| Field      | Value                     |
| ---------- | ------------------------- |
| Capability | `environment.get_network` |

## `environment.openExternalUrl`

| Field      | Value                           |
| ---------- | ------------------------------- |
| Capability | `environment.open_external_url` |

## `environment.theme.getCurrent`

| Field      | Value                           |
| ---------- | ------------------------------- |
| Capability | `environment.theme.get_current` |

## `wallet.checkAddress`

| Field      | Value                  |
| ---------- | ---------------------- |
| Capability | `wallet.check_address` |

## `wallet.filterUnlockedCoins`

| Field      | Value                          |
| ---------- | ------------------------------ |
| Capability | `wallet.filter_unlocked_coins` |

## `wallet.getAssetBalance`

| Field      | Value                      |
| ---------- | -------------------------- |
| Capability | `wallet.get_asset_balance` |

## `wallet.getAssetCoins`

| Field      | Value                    |
| ---------- | ------------------------ |
| Capability | `wallet.get_asset_coins` |

## `wallet.getCoins`

| Field      | Value              |
| ---------- | ------------------ |
| Capability | `wallet.get_coins` |

## `wallet.getCoinsByIds`

| Field      | Value                     |
| ---------- | ------------------------- |
| Capability | `wallet.get_coins_by_ids` |

## `wallet.getDerivations`

| Field      | Value                    |
| ---------- | ------------------------ |
| Capability | `wallet.get_derivations` |

## `wallet.getKey`

| Field      | Value            |
| ---------- | ---------------- |
| Capability | `wallet.get_key` |

## `wallet.getPendingTransactions`

| Field      | Value                             |
| ---------- | --------------------------------- |
| Capability | `wallet.get_pending_transactions` |

## `wallet.getPublicKeys`

| Field      | Value                    |
| ---------- | ------------------------ |
| Capability | `wallet.get_public_keys` |

## `wallet.getSecretKey`

| Field      | Value                   |
| ---------- | ----------------------- |
| Capability | `wallet.get_secret_key` |

## `wallet.getSpendableCoinCount`

| Field      | Value                             |
| ---------- | --------------------------------- |
| Capability | `wallet.get_spendable_coin_count` |

## `wallet.getSyncStatus`

| Field      | Value                    |
| ---------- | ------------------------ |
| Capability | `wallet.get_sync_status` |

## `wallet.getTransaction`

| Field      | Value                    |
| ---------- | ------------------------ |
| Capability | `wallet.get_transaction` |

## `wallet.getTransactions`

| Field      | Value                     |
| ---------- | ------------------------- |
| Capability | `wallet.get_transactions` |

## `wallet.getVersion`

| Field      | Value                |
| ---------- | -------------------- |
| Capability | `wallet.get_version` |

## `wallet.getXchUsdPrice`

| Field      | Value                      |
| ---------- | -------------------------- |
| Capability | `wallet.get_xch_usd_price` |

## `wallet.sendTransaction`

| Field      | Value                     |
| ---------- | ------------------------- |
| Capability | `wallet.send_transaction` |

## `wallet.sendXch`

| Field      | Value             |
| ---------- | ----------------- |
| Capability | `wallet.send_xch` |

## `wallet.signCoinSpends`

| Field      | Value                     |
| ---------- | ------------------------- |
| Capability | `wallet.sign_coin_spends` |

## `wallet.signMessage`

| Field      | Value                 |
| ---------- | --------------------- |
| Capability | `wallet.sign_message` |
