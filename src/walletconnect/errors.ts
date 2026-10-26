/**
 * An error with a machine-readable code, which is passed to the site in the
 * data of the JSON-RPC error response, as `{"code": ...}`, so that it doesn't
 * have to parse the message.
 */
export class WalletConnectError extends Error {
  constructor(
    message: string,
    readonly code: string,
  ) {
    super(message);
    this.name = 'WalletConnectError';
  }
}
