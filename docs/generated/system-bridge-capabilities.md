# System bridge capabilities

## `runtime_manager.list_runtimes`

**List app runtimes**

Allows the system app to inspect running Sage app runtimes.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `runtime_manager.focus_runtime`

**Focus app runtimes**

Allows the system app to focus running Sage app runtimes.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `runtime_manager.hide_runtime`

**Hide app runtimes**

Allows the system app to hide running Sage app runtimes.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `runtime_manager.kill_runtime`

**Kill app runtimes**

Allows the system app to stop running Sage app runtimes.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `runtime_manager.listen_runtimes_changed`

**Observe runtime changes**

Allows the system app to receive events when Sage app runtimes change.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `capability_definitions.read`

**Read capability definitions**

Allows the system app to read Sage capability definitions.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `app_permissions.read`

**Read app permissions**

Allows the system app to read app permissions for review.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `app_permissions.apply`

**Apply app permissions**

Allows the system app to apply reviewed app permission changes.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `app_update.read`

**Read app update review context**

Allows the system app to read update information for installed Sage apps.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `app_update.apply`

**Apply app updates**

Allows the system app to download and apply approved Sage app updates.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

