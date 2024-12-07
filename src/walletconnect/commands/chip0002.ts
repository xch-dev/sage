import { commands } from '@/bindings';
import { BigNumber } from 'bignumber.js';
import { Params } from '../commands';

export async function handleChainId(_params: Params<'chip0002_chainId'>) {
  const result = await commands.networkConfig();

  if (result.status === 'error') {
    throw new Error(result.error.reason);
  }

  return result.data.network_id;
}

export async function handleConnect(_params: Params<'chip0002_connect'>) {
  // This command is only supported for compatibility with the CHIP-0002 spec.
  // It does not need to do anything.
  return true;
}

export async function handleGetPublicKeys(
  params: Params<'chip0002_getPublicKeys'>,
) {
  const result = await commands.getDerivations({
    limit: params?.limit ?? 10,
    offset: params?.offset ?? 0,
  });

  if (result.status === 'error') {
    throw new Error(result.error.reason);
  }

  return result.data.derivations.map((derivation) => derivation.public_key);
}

export async function handleFilterUnlockedCoins(
  params: Params<'chip0002_filterUnlockedCoins'>,
) {
  const result = await commands.filterUnlockedCoins({
    coin_ids: params.coinNames,
  });

  if (result.status === 'error') {
    throw new Error(result.error.reason);
  }

  return result.data;
}

export async function handleGetAssetCoins(
  params: Params<'chip0002_getAssetCoins'>,
) {
  const result = await commands.getAssetCoins(params);

  if (result.status === 'error') {
    throw new Error(result.error.reason);
  }

  return result.data;
}

export async function handleGetAssetBalance(
  params: Params<'chip0002_getAssetBalance'>,
) {
  const result = await commands.getAssetCoins({
    ...params,
    includedLocked: true,
  });

  if (result.status === 'error') {
    throw new Error(result.error.reason);
  }

  let confirmed = BigNumber(0);
  let spendable = BigNumber(0);
  let spendableCoinCount = 0;

  for (const record of result.data) {
    confirmed = confirmed.plus(record.coin.amount);

    if (!record.locked) {
      spendable = spendable.plus(record.coin.amount);
      spendableCoinCount += 1;
    }
  }

  return {
    confirmed: confirmed.toString(),
    spendable: spendable.toString(),
    spendableCoinCount,
  };
}

export async function handleSignCoinSpends(
  params: Params<'chip0002_signCoinSpends'>,
) {
  const result = await commands.signCoinSpends({
    coin_spends: params.coinSpends.map((coinSpend) => {
      return {
        coin: {
          parent_coin_info: coinSpend.coin.parent_coin_info,
          puzzle_hash: coinSpend.coin.puzzle_hash,
          amount: coinSpend.coin.amount.toString(),
        },
        puzzle_reveal: coinSpend.puzzle_reveal,
        solution: coinSpend.solution,
      };
    }),
    partial: params.partialSign,
    auto_submit: false,
  });

  if (result.status === 'error') {
    throw new Error(result.error.reason);
  }

  return result.data.spend_bundle.aggregated_signature;
}

export async function handleSignMessage(
  params: Params<'chip0002_signMessage'>,
) {
  const result = await commands.signMessageWithPublicKey(params);

  if (result.status === 'error') {
    throw new Error(result.error.reason);
  }

  return result.data.signature;
}

export async function handleSendTransaction(
  params: Params<'chip0002_sendTransaction'>,
) {
  const result = await commands.sendTransactionImmediately({
    spend_bundle: params.spendBundle,
  });

  if (result.status === 'error') {
    throw new Error(result.error.reason);
  }

  return result.data;
}
