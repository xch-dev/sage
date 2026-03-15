import { commands } from '@/bindings';
import { Params } from '../commands';
import { HandlerContext } from '../handler';

export async function handleCreateOffer(
  params: Params<'chia_createOffer'>,
  context: HandlerContext,
) {
  const password = await context.requestPassword(context.hasPassword);
  if (password === null && context.hasPassword)
    throw new Error('Authentication failed');

  if (!(await context.promptIfEnabled())) {
    throw new Error('Authentication failed');
  }

  const data = await commands.makeOffer({
    fee: params.fee ?? 0,
    offered_assets: params.offerAssets.map((asset) => ({
      asset_id: asset.assetId === '' ? null : asset.assetId,
      hidden_puzzle_hash: asset.hiddenPuzzleHash,
      amount: asset.amount,
    })),
    requested_assets: params.requestAssets.map((asset) => ({
      asset_id: asset.assetId === '' ? null : asset.assetId,
      hidden_puzzle_hash: asset.hiddenPuzzleHash,
      amount: asset.amount,
    })),
    expires_at_second: null,
    password,
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
  const password = await context.requestPassword(context.hasPassword);
  if (password === null && context.hasPassword)
    throw new Error('Authentication failed');

  if (!(await context.promptIfEnabled())) {
    throw new Error('Authentication failed');
  }

  const data = await commands.takeOffer({
    offer: params.offer,
    fee: params.fee ?? 0,
    auto_submit: true,
    password,
  });

  return { id: data.transaction_id };
}

export async function handleCancelOffer(
  params: Params<'chia_cancelOffer'>,
  context: HandlerContext,
) {
  const password = await context.requestPassword(context.hasPassword);
  if (password === null && context.hasPassword)
    throw new Error('Authentication failed');

  if (!(await context.promptIfEnabled())) {
    throw new Error('Authentication failed');
  }

  await commands.cancelOffer({
    offer_id: params.id,
    fee: params.fee ?? 0,
    auto_submit: true,
    password,
  });

  return {};
}
