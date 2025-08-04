import { commands } from '@/bindings';
import { Params } from '../commands';
import { HandlerContext } from '../handler';

export async function handleCreateOffer(
  params: Params<'chia_createOffer'>,
  context: HandlerContext,
) {
  if (!(await context.promptIfEnabled())) {
    throw new Error('Authentication failed');
  }

  const data = await commands.makeOffer({
    fee: params.fee ?? 0,
    offered_assets: params.offerAssets.map((asset) => ({
      asset_id: asset.assetId === '' ? null : asset.assetId,
      amount: asset.amount,
    })),
    requested_assets: params.requestAssets.map((asset) => ({
      asset_id: asset.assetId === '' ? null : asset.assetId,
      amount: asset.amount,
    })),
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
