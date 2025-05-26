# Transaction Failure Event System

This document describes the implementation of the transaction failure event system that provides real-time notifications when transactions fail.

## Overview

The system consists of three main components:

1. **Rust Backend**: Emits `TransactionFailed` events when transactions fail
2. **TypeScript Hook**: Listens for these events and handles them
3. **Toast Notifications**: Shows user-friendly error messages

## Implementation Details

### Rust Side

#### Event Definition

```rust
// crates/sage-api/src/events.rs
pub enum SyncEvent {
    // ... other events
    TransactionFailed { transaction_id: String },
}
```

#### Event Emission

```rust
// crates/sage-wallet/src/queues/transaction_queue.rs
self.sync_sender
    .send(SyncEvent::TransactionEnded {
        transaction_id,
        success: false,
    })
    .await
    .ok();
```

#### Event Mapping

```rust
// src-tauri/src/app_state.rs
SyncEvent::TransactionEnded { transaction_id, success } => {
    if success {
        ApiEvent::CoinState
    } else {
        ApiEvent::TransactionFailed { 
            transaction_id: transaction_id.to_string() 
        }
    }
}
```

### TypeScript Side

#### Hook Implementation

```typescript
// src/hooks/useTransactionFailures.ts
export function useTransactionFailures() {
  useEffect(() => {
    const unlisten = events.syncEvent.listen((event) => {
      if (event.payload.type === 'transaction_failed') {
        const { transaction_id } = event.payload;
        
        // Log error to console
        console.error('Transaction failed:', {
          transaction_id,
          timestamp: new Date().toISOString(),
        });
        
        // Show toast notification
        toast.error(
          t`Transaction failed: ${transaction_id.slice(0, 8)}...`,
          {
            autoClose: 8000,
            toastId: `transaction-failed-${transaction_id}`,
          }
        );
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, []);
}
```

#### Global Integration

```typescript
// src/App.tsx
function AppInner() {
  // ... other hooks
  useTransactionFailures(); // Enable global transaction failure handling
  
  // ... rest of component
}
```

## Features

### Toast Notifications

- **Duration**: 8 seconds (longer than default to ensure users see the error)
- **Deduplication**: Uses transaction ID to prevent duplicate toasts
- **User-friendly**: Shows shortened transaction ID for readability
- **Styling**: Uses error styling (red) to indicate failure

### Console Logging

- **Detailed Information**: Logs full transaction ID and timestamp
- **Debugging**: Helps developers debug transaction failures
- **Persistence**: Console logs remain available for investigation

### Event Handling

- **Real-time**: Events are processed immediately when transactions fail
- **Global**: Works across the entire application
- **Automatic**: No manual setup required per component

## Testing

A test component is available in the Settings page under the Advanced tab:

1. Navigate to Settings
2. Click on the "Advanced" tab
3. Find the "Development & Testing" section
4. Click "Trigger Test Transaction Failure"

This will emit a test event to verify the system works correctly.

## Usage

The system is automatically active once the application starts. No additional setup is required.

### For Developers

To add custom handling for transaction failures in specific components:

```typescript
import { events } from '../bindings';

useEffect(() => {
  const unlisten = events.syncEvent.listen((event) => {
    if (event.payload.type === 'transaction_failed') {
      // Custom handling here
      const { transaction_id } = event.payload;
      // ... your custom logic
    }
  });

  return () => {
    unlisten.then((u) => u());
  };
}, []);
```

## Error Information

When a transaction fails, the following information is available:

- **Transaction ID**: Full 32-byte transaction identifier
- **Timestamp**: When the failure was detected
- **Event Type**: Always `'transaction_failed'`

## Future Enhancements

Potential improvements could include:

1. **Retry Mechanism**: Allow users to retry failed transactions
2. **Failure Reasons**: Include specific error messages from the network
3. **Transaction Details**: Show more context about what the transaction was trying to do
4. **User Actions**: Provide actionable steps for common failure scenarios
