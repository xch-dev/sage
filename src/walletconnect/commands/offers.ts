import { Assets, commands } from '@/bindings';
import { BigNumber } from 'bignumber.js';
import { Params } from '../commands';
import { HandlerContext } from '../handler';

export async function handleCreateOffer(
  params: Params<'chia_createOffer'>,
  context: HandlerContext,
) {
  if (!(await context.promptIfEnabled())) {
    throw new Error('Authentication failed');
  }

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

  const data = await commands.makeOffer({
    fee: params.fee ?? 0,
    offered_assets: offerAssets,
    requested_assets: requestAssets,
    expires_at_second: null,
  });

  return {
    offer: data.offer,
    id: data.offer_id,
  };
}

export async function handleTakeOffer(
  params: Params<'chia_takeOffer'>,
  context: HandlerContext,
) {
  if (!(await context.promptIfEnabled())) {
    throw new Error('Authentication failed');
  }

  const data = await commands.takeOffer({
    offer: params.offer,
    fee: params.fee ?? 0,
    auto_submit: true,
  });

  return { id: data.transaction_id };
}

export async function handleCancelOffer(
  params: Params<'chia_cancelOffer'>,
  context: HandlerContext,
) {
  if (!(await context.promptIfEnabled())) {
    throw new Error('Authentication failed');
  }

  await commands.cancelOffer({
    offer_id: params.id,
    fee: params.fee ?? 0,
    auto_submit: true,
  });

  return {};
}
