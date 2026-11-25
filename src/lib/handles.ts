import {
  Error as BackendError,
  commands,
  HandleConfirmation,
  RecipientError,
} from '@/bindings';
import { t } from '@lingui/core/macro';
import { create } from 'zustand';

const HANDLE_PATTERN = /^@[a-z0-9]{3,63}$/;

/**
 * Whether a recipient is written as an XCHandles handle. The backend resolves
 * these even when this build hides XCHandles, so they're always previewed and
 * validated as handles, rather than only when XCHandles is shown.
 */
export function isHandleInput(value: string): boolean {
  return value.trim().startsWith('@');
}

/** Explains why resolving or verifying a handle failed. */
export function handleErrorMessage(error: BackendError): string {
  const reason = error.reason;

  switch (error.handle_error_code) {
    case 'not_found':
    case 'expired':
      return t`This handle isn't registered, or has expired`;
    case 'unavailable':
      return t`This handle couldn't be verified right now: ${reason}`;
    default:
      return t`This handle couldn't be verified: ${reason}`;
  }
}

/** Whether a handle is written as `@handle` with a valid name. */
export function isValidHandle(value: string): boolean {
  return HANDLE_PATTERN.test(value.trim());
}

/**
 * Checks that a recipient is a valid address for this network, or a
 * syntactically valid handle. Whether the handle is registered is checked by
 * the backend when the transaction is created.
 */
export async function validateRecipient(value: string): Promise<boolean> {
  const recipient = value.trim();

  if (isHandleInput(recipient)) {
    return isValidHandle(recipient);
  }

  return await commands.validateAddress(recipient);
}

function recipientErrors(error: unknown): RecipientError[] | null {
  if (
    typeof error === 'object' &&
    error !== null &&
    'kind' in error &&
    error.kind === 'recipient' &&
    'recipients' in error &&
    Array.isArray(error.recipients)
  ) {
    return error.recipients as RecipientError[];
  }

  return null;
}

export interface HandleChange {
  handle: string;
  previousAddress: string;
  address: string;
}

interface HandleChangeState {
  changes: HandleChange[] | null;
  resolve: ((confirmed: boolean) => void) | null;
  confirm: (changes: HandleChange[]) => Promise<boolean>;
  finish: (confirmed: boolean) => void;
}

export const useHandleChangeStore = create<HandleChangeState>((set, get) => ({
  changes: null,
  resolve: null,
  confirm: (changes) =>
    new Promise<boolean>((resolve) => {
      // Only one request is reviewed at a time, so an earlier one is declined.
      get().resolve?.(false);
      set({ changes, resolve });
    }),
  finish: (confirmed) => {
    get().resolve?.(confirmed);
    set({ changes: null, resolve: null });
  },
}));

/**
 * Thrown when the user declines to send to a handle which changed. It keeps the
 * recipient errors, so that dApps can be told which handles changed.
 */
export interface HandleChangeDeclined {
  kind: 'cancelled';
  reason: string;
  recipients: RecipientError[];
}

/**
 * Runs a request which may take handle recipients. If a handle now resolves to
 * a different address than before, the user is shown both addresses, and the
 * request is only made again with the new addresses confirmed if they accept.
 *
 * The backend makes the final check, both here and again when the transaction
 * is submitted, so this can't send anywhere the user didn't confirm.
 */
export async function withHandleConfirmation<T>(
  request: (confirmedHandles: HandleConfirmation[]) => Promise<T>,
): Promise<T> {
  let confirmed: HandleConfirmation[] = [];

  // A handle could change again while the user is reviewing it, so this is
  // repeated until nothing has changed, or the user declines.
  for (;;) {
    try {
      return await request(confirmed);
    } catch (error) {
      const recipients = recipientErrors(error);

      if (
        !recipients ||
        recipients.length === 0 ||
        !recipients.every(
          (recipient) =>
            recipient.kind === 'changed' &&
            recipient.handle &&
            recipient.address &&
            recipient.previous_address,
        )
      ) {
        throw error;
      }

      const changes = recipients.map((recipient) => ({
        handle: recipient.handle as string,
        previousAddress: recipient.previous_address as string,
        address: recipient.address as string,
      }));

      if (!(await useHandleChangeStore.getState().confirm(changes))) {
        const declined: HandleChangeDeclined = {
          kind: 'cancelled',
          reason:
            'The user did not confirm the new address of a changed handle',
          recipients,
        };
        throw declined;
      }

      confirmed = [
        ...confirmed.filter(
          (item) => !changes.some((change) => change.handle === item.handle),
        ),
        ...changes.map((change) => ({
          handle: change.handle,
          address: change.address,
        })),
      ];
    }
  }
}
