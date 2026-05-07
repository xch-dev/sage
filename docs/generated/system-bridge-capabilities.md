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

## `runtime_manager.focus_taskbar_runtime`

**Focus taskbar app runtime**

Allows the system app to focus running Sage taskbar app runtime.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `runtime_manager.hide_runtime`

**Hide app runtime**

Allows the system app to hide running Sage app runtime.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `runtime_manager.kill_runtime`

**Kill app runtime**

Allows the system app to stop running Sage app runtime.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `runtime_manager.get_active_taskbar_runtime`

**Get active runtime**

Allows the system app to retrieve the currently active Sage app runtime.

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

## `runtime_manager.listen_active_runtime_changed`

**Observe active runtime changes**

Allows the system app to receive events when the active Sage app runtime changes.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `runtime_manager.hide_self`

**Hide itself**

Allows the system app to hide its own runtime.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `runtime_manager.close_self`

**Close itself**

Allows the system app to close its own runtime.

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

## `app_install.preview`

**Preview app installs**

Allows the system app to preview URL and ZIP app installations.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `app_install.apply`

**Install apps**

Allows the system app to install Sage apps after review.

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

## `file_system.select_file`

**Select file**

Allows the system app to ask the user to select a local file.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `bridge_approval.list`

**List bridge approvals**

Allows the system app to list pending bridge approvals.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `bridge_approval.resolve`

**Resolve bridge approval**

Allows the system app to resolve a pending bridge approval.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `bridge_approval.listen_changed`

**Listen for bridge approval changes**

Allows the system app to listen for changes in pending bridge approvals.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

## `donation.get_details`

**Get details for donation**

Allows the system app to retrieve details to send donation.

| Flag | Value |
|---|---|
| Requestable by app | `true` |
| User grantable | `false` |
| Shared with app | `true` |
| Externally observable | `false` |
| Accesses sensitive secret | `false` |

