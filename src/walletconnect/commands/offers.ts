import { Assets, commands } from '@/bindings';
import { useWalletState } from '@/state';
import { BigNumber } from 'bignumber.js';
import { Params } from '../commands';

export async function handleCreateOffer(params: Params<'chia_createOffer'>) {
  const state = useWalletState.getState();

  const defaultAssets = (): Assets => {
    return {
      xch: '0',
      cats: [],
      nfts: [],
    };
  };

  const offerAssets = defaultAssets();
  const requestAssets = defaultAssets();

  for (const [from, to] of [
    [params.offerAssets, offerAssets],
    [params.requestAssets, requestAssets],
  ] as const) {
    for (const item of from) {
      if (item.assetId.startsWith('nft')) {
        to.nfts.push(item.assetId);
      } else if (item.assetId === '') {
        to.xch = BigNumber(to.xch).plus(BigNumber(item.amount)).toString();
      } else {
        const catAmount = BigNumber(item.amount);
        const found = to.cats.find((cat) => cat.asset_id === item.assetId);

        if (found) {
          found.amount = BigNumber(found.amount).plus(catAmount).toString();
        } else {
          to.cats.push({
            asset_id: item.assetId,
            amount: catAmount.toString(),
          });
        }
      }
    }
  }

  const result = await commands.makeOffer({
    fee: params.fee ?? 0,
    offered_assets: offerAssets,
    requested_assets: requestAssets,
    expires_at_second: null,
  });

  if (result.status === 'error') {
    throw new Error(result.error.reason);
  }

  return {
    offer: result.data.offer,
    id: result.data.offer_id,
  };
}

export async function handleTakeOffer(params: Params<'chia_takeOffer'>) {
  const result = await commands.takeOffer({
    offer: params.offer,
    fee: params.fee ?? 0,
    auto_submit: true,
  });

  if (result.status === 'error') {
    throw new Error(result.error.reason);
  }

  return { id: result.data.transaction_id };
}
