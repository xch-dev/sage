import { parseCommand, WalletConnectCommand } from './commands';
import {
  handleChainId,
  handleConnect,
  handleFilterUnlockedCoins,
  handleGetAssetBalance,
  handleGetAssetCoins,
  handleGetPublicKeys,
  handleSendTransaction,
  handleSignCoinSpends,
  handleSignMessage,
} from './commands/chip0002';
import {
  handleBulkMintNfts,
  handleGetAddress,
  handleGetNfts,
  handleSend,
  handleSignMessageByAddress,
} from './commands/high-level';
import {
  handleCancelOffer,
  handleCreateOffer,
  handleTakeOffer,
} from './commands/offers';
import { t } from '@lingui/core/macro';

export interface HandlerContext {
  requestPassword: (hasPassword: boolean) => Promise<string | null | undefined>;
  hasPassword: boolean;
  /** True when the active wallet has no signing keys (cold/watch-only). */
  isReadOnly: boolean;
}

const SIGNING_COMMANDS = new Set<WalletConnectCommand>([
  'chip0002_signCoinSpends',
  'chip0002_signMessage',
  'chia_createOffer',
  'chia_takeOffer',
  'chia_cancelOffer',
  'chia_send',
  'chia_signMessageByAddress',
  'chia_bulkMintNfts',
]);

export const handleCommand = async (
  command: WalletConnectCommand,
  params: unknown,
  context: HandlerContext,
) => {
  if (context.isReadOnly && SIGNING_COMMANDS.has(command)) {
    throw new Error(
      t`This wallet is read-only and cannot sign, so ${command} is not available`,
    );
  }

  switch (command) {
    case 'chip0002_connect':
      return await handleConnect();
    case 'chip0002_chainId':
      return await handleChainId();
    case 'chip0002_getPublicKeys':
      return await handleGetPublicKeys(parseCommand(command, params));
    case 'chip0002_filterUnlockedCoins':
      return await handleFilterUnlockedCoins(parseCommand(command, params));
    case 'chip0002_getAssetCoins':
      return await handleGetAssetCoins(parseCommand(command, params));
    case 'chip0002_getAssetBalance':
      return await handleGetAssetBalance(parseCommand(command, params));
    case 'chip0002_signCoinSpends':
      return await handleSignCoinSpends(parseCommand(command, params), context);
    case 'chip0002_signMessage':
      return await handleSignMessage(parseCommand(command, params), context);
    case 'chip0002_sendTransaction':
      return await handleSendTransaction(parseCommand(command, params));
    case 'chia_createOffer':
      return await handleCreateOffer(parseCommand(command, params), context);
    case 'chia_takeOffer':
      return await handleTakeOffer(parseCommand(command, params), context);
    case 'chia_cancelOffer':
      return await handleCancelOffer(parseCommand(command, params), context);
    case 'chia_getNfts':
      return await handleGetNfts(parseCommand(command, params));
    case 'chia_send':
      return await handleSend(parseCommand(command, params), context);
    case 'chia_getAddress':
      return await handleGetAddress();
    case 'chia_signMessageByAddress':
      return await handleSignMessageByAddress(
        parseCommand(command, params),
        context,
      );
    case 'chia_bulkMintNfts':
      return await handleBulkMintNfts(parseCommand(command, params), context);
    default:
      throw new Error(`Unknown command: ${command}`);
  }
};
