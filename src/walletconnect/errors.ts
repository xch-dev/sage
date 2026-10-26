/**
 * An error with a machine-readable code, which is passed to the site as the
 * data of the JSON-RPC error response so that it doesn't have to parse the
 * message.
 */
export class WalletConnectError extends Error {
  constructor(
    message: string,
    readonly data: string,
  ) {
    super(message);
    this.name = 'WalletConnectError';
  }
}
