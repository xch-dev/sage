import type { RustBridgeApprovalRequest } from 'sage-system-app-sdk';

type SendXchApproval = Extract<RustBridgeApprovalRequest, { kind: 'sendXch' }>;

export interface SendXchHandleState {
  handle: string;
  /** The verified address the handle resolves to. */
  address: string | null;
  /** Whether the address comes from an earlier verification. */
  cached: boolean;
  /** The address it resolved to before, if it changed since it was last used. */
  previousAddress: string | null;
  error: string | null;
}

/**
 * Sage resolves a handle recipient when the approval is requested. A handle
 * which now resolves to a different address than before, or than the one last
 * sent to, may have changed hands, and has to be acknowledged.
 */
export function sendXchHandleState(
  approval: SendXchApproval,
): SendXchHandleState | null {
  if (!approval.handle) return null;

  const record = approval.handle.record ?? null;

  const previousAddress = record
    ? (record.previous_address ??
      (record.last_sent_address && record.last_sent_address !== record.address
        ? record.last_sent_address
        : null))
    : null;

  return {
    handle: approval.summary.address,
    address: record?.address ?? null,
    cached: record?.cached ?? false,
    previousAddress,
    error:
      approval.handle.error ??
      (record ? null : 'The handle could not be resolved.'),
  };
}
