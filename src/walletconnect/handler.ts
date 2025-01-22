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

export const handleCommand = async (
  command: WalletConnectCommand,
  params: unknown,
) => {
  switch (command) {
    case 'chip0002_connect':
      return await handleConnect(parseCommand(command, params));
    case 'chip0002_chainId':
      return await handleChainId(parseCommand(command, params));
    case 'chip0002_getPublicKeys':
      return await handleGetPublicKeys(parseCommand(command, params));
    case 'chip0002_filterUnlockedCoins':
      return await handleFilterUnlockedCoins(parseCommand(command, params));
    case 'chip0002_getAssetCoins':
      return await handleGetAssetCoins(parseCommand(command, params));
    case 'chip0002_getAssetBalance':
      return await handleGetAssetBalance(parseCommand(command, params));
    case 'chip0002_signCoinSpends':
      return await handleSignCoinSpends(parseCommand(command, params));
    case 'chip0002_signMessage':
      return await handleSignMessage(parseCommand(command, params));
    case 'chip0002_sendTransaction':
      return await handleSendTransaction(parseCommand(command, params));
    case 'chia_createOffer':
      return await handleCreateOffer(parseCommand(command, params));
    case 'chia_takeOffer':
      return await handleTakeOffer(parseCommand(command, params));
    case 'chia_cancelOffer':
      return await handleCancelOffer(parseCommand(command, params));
    case 'chia_getNfts':
      return await handleGetNfts(parseCommand(command, params));
    case 'chia_send':
      return await handleSend(parseCommand(command, params));
    case 'chia_getAddress':
      return await handleGetAddress(parseCommand(command, params));
    case 'chia_signMessageByAddress':
      return await handleSignMessageByAddress(parseCommand(command, params));
    default:
      throw new Error(`Unknown command: ${command}`);
  }
};
