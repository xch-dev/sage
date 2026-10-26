import { commands, Error as SageError, HandleRecord } from '@/bindings';
import { xchandlesEnabled } from '@/lib/features';
import { t } from '@lingui/core/macro';
import { Params, Return } from '../commands';
import { WalletConnectError } from '../errors';

type WalletConnectHandle = Return<'chia_resolveHandle'>;

// Resolving a handle contacts the XCHandles API and peers while holding the
// wallet, so sites may only resolve a limited number of handles per minute.
export const RESOLVE_LIMIT = 20;
const RESOLVE_WINDOW_MS = 60_000;
const recentResolutions: number[] = [];

function checkResolveRateLimit() {
  const now = Date.now();

  while (
    recentResolutions.length > 0 &&
    recentResolutions[0] <= now - RESOLVE_WINDOW_MS
  ) {
    recentResolutions.shift();
  }

  if (recentResolutions.length >= RESOLVE_LIMIT) {
    throw new WalletConnectError(
      t`Too many handle lookups, try again in a minute`,
      'rate_limited',
    );
  }

  recentResolutions.push(now);
}

function isSageError(error: unknown): error is SageError {
  return typeof error === 'object' && error !== null && 'reason' in error;
}

// Passes XCHandles failures on with a machine-readable code, so that sites can
// tell an invalid or expired handle from an API that is temporarily unavailable.
async function withHandleErrors<T>(request: () => Promise<T>): Promise<T> {
  if (!xchandlesEnabled) {
    throw new Error(t`XCHandles is not enabled in this build of Sage`);
  }

  try {
    return await request();
  } catch (error) {
    if (isSageError(error) && error.handle_error_code) {
      throw new WalletConnectError(error.reason, error.handle_error_code);
    }
    throw error;
  }
}

function toRecord(record: HandleRecord): WalletConnectHandle {
  return {
    handle: record.handle,
    handleHash: record.handle_hash,
    address: record.address,
    status: record.status,
    previousAddress: record.previous_address,
    lastSentAddress: record.last_sent_address,
    cached: record.cached,
    verification: record.verification,
    verificationReason: record.verification_reason,
    ownerLauncherId: record.owner_launcher_id,
    resolvedLauncherId: record.resolved_launcher_id,
    expiration: record.expiration,
    slotCoinId: record.slot_coin_id,
    slotConfirmationHeight: record.slot_confirmation_height,
    resolvedCoinId: record.resolved_coin_id,
    verifiedHeight: record.verified_height,
    verifiedTimestamp: record.verified_timestamp,
    lastVerifiedAt: record.last_verified_at,
  };
}

export async function handleResolveHandle(
  params: Params<'chia_resolveHandle'>,
): Promise<Return<'chia_resolveHandle'>> {
  return await withHandleErrors(async () => {
    checkResolveRateLimit();
    const data = await commands.resolveHandle({ handle: params.handle });
    return toRecord(data.handle);
  });
}

export async function handleGetHandle(
  params: Params<'chia_getHandle'>,
): Promise<Return<'chia_getHandle'>> {
  return await withHandleErrors(async () => {
    checkResolveRateLimit();
    const data = await commands.getHandle({ handle: params.handle });
    const transfer = data.pending_transfer;

    return {
      handle: toRecord(data.handle),
      slot: {
        registryLauncherId: data.slot.registry_launcher_id,
        counter: data.slot.counter,
        leftHandleHash: data.slot.left_handle_hash,
        rightHandleHash: data.slot.right_handle_hash,
        parentId: data.slot.parent_id,
        parentParentId: data.slot.parent_parent_id,
        parentInnerPuzzleHash: data.slot.parent_inner_puzzle_hash,
        resolvedConfirmationHeight: data.slot.resolved_confirmation_height,
        indexedHeight: data.slot.indexed_height,
      },
      pendingTransferStatus: data.pending_transfer_status,
      pendingTransfer: transfer
        ? {
            newOwnerLauncherId: transfer.new_owner_launcher_id,
            newResolvedLauncherId: transfer.new_resolved_launcher_id,
            minimumExecutionHeight: transfer.minimum_execution_height,
            initiatorCoinId: transfer.initiator_coin_id,
            executorCoinId: transfer.executor_coin_id,
            updateConfirmationHeight: transfer.update_confirmation_height,
            updateSlotCoinId: transfer.update_slot_coin_id,
          }
        : null,
    };
  });
}

export async function handleGetOwnedHandles(
  params: Params<'chia_getOwnedHandles'>,
): Promise<Return<'chia_getOwnedHandles'>> {
  return await withHandleErrors(async () => {
    // Refreshing would let a site make the wallet look up handles, so only
    // handles that were already verified are listed.
    const data = await commands.getHandles({
      filter: 'owned',
      include_unverified: false,
      refresh: false,
      offset: params?.offset ?? 0,
      limit: params?.limit ?? 50,
    });

    return {
      handles: data.handles.map(toRecord),
      total: data.total,
    };
  });
}
