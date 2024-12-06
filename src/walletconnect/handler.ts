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
import { handleCreateOffer, handleTakeOffer } from './commands/offers';

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
    default:
      throw new Error(`Unknown command: ${command}`);
  }
};
