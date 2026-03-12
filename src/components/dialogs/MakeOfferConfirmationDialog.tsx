import { commands, NftRecord, OptionRecord, TokenRecord } from '@/bindings';
import { AssetIcon } from '@/components/AssetIcon';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { marketplaces } from '@/lib/marketplaces';
import { emptyNftRecord, getAssetDisplayName } from '@/lib/utils';
import { Assets, OfferState, TokenAmount } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import BigNumber from 'bignumber.js';
import { AlertTriangle } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { NumberFormat } from '../NumberFormat';
import { ScrollArea } from '../ui/scroll-area';

interface MakeOfferConfirmationDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onConfirm: () => void;
  offerState: OfferState;
  splitNftOffers: boolean;
  fee: string;
  enabledMarketplaces?: Record<string, boolean>;
  setEnabledMarketplaces?: (marketplaces: Record<string, boolean>) => void;
}
interface TokenWithName extends TokenAmount {
  displayName?: string;
  iconUrl?: string | null;
  precision?: number;
}

function AssetDisplay({
  assets,
  type,
}: {
  assets: Assets;
  type: 'offered' | 'requested';
}) {
  const [nftDetailsList, setNftDetailsList] = useState<NftRecord[]>([]);
  const [loadingNfts, setLoadingNfts] = useState(false);
  const [optionDetailsList, setOptionDetailsList] = useState<OptionRecord[]>(
    [],
  );
  const [loadingOptions, setLoadingOptions] = useState(false);
  const [tokensWithNames, setTokensWithNames] = useState<TokenWithName[]>([]);
  const [loadingTokens, setLoadingTokens] = useState(false);

  // Create a stable reference for IDs to prevent infinite re-renders
  const nftIds = useMemo(() => {
    if (!assets.nfts || assets.nfts.length === 0) return [];
    return assets.nfts.filter((id) => id && typeof id === 'string');
  }, [assets.nfts]);

  const optionIds = useMemo(() => {
    if (!assets.options || assets.options.length === 0) return [];
    return assets.options.filter((id) => id && typeof id === 'string');
  }, [assets.options]);

  useEffect(() => {
    const fetchTokenNames = async () => {
      if (assets.tokens.length === 0) {
        setTokensWithNames([]);
        return;
      }
      setLoadingTokens(true);

      try {
        const tokensWithNamesPromises = assets.tokens.map(
          async ({ asset_id: assetId, amount, fee_policy }) => {
            try {
              const tokenResponse = await commands.getToken({
                asset_id: assetId,
              });
              const token = tokenResponse.token;
              if (token) {
                return {
                  asset_id: assetId,
                  amount,
                  displayName: getAssetDisplayName(
                    token.name,
                    token.ticker,
                    'token',
                  ),
                  iconUrl: token.icon_url,
                  precision: token.precision,
                  fee_policy,
                };
              } else {
                return {
                  asset_id: assetId,
                  amount,
                  displayName: getAssetDisplayName(null, null, 'token'),
                  iconUrl: null,
                  precision: 3,
                  fee_policy,
                };
              }
            } catch (error) {
              console.error(
                `Failed to fetch token info for ${assetId}:`,
                error,
              );
              return {
                asset_id: assetId,
                amount,
                displayName: getAssetDisplayName(null, null, 'token'),
                iconUrl: null,
                precision: 3,
                fee_policy,
              };
            }
          },
        );

        const tokensWithNamesResults = await Promise.all(
          tokensWithNamesPromises,
        );
        setTokensWithNames(tokensWithNamesResults);
      } catch (error) {
        console.error('Error fetching token names:', error);
        // Fallback to original tokens without names
        setTokensWithNames(
          assets.tokens.map((token) => ({
            ...token,
            displayName: getAssetDisplayName(null, null, 'token'),
            iconUrl: null,
          })),
        );
      }
      setLoadingTokens(false);
    };

    fetchTokenNames();
  }, [assets.tokens]);

  useEffect(() => {
    const fetchNftDetails = async () => {
      setLoadingNfts(true);

      const displayableNfts: NftRecord[] = [];
      for (const nftId of nftIds) {
        try {
          const nftRecordResponse = await commands.getNft({ nft_id: nftId });
          if (nftRecordResponse.nft) {
            displayableNfts.push(nftRecordResponse.nft);
          } else {
            displayableNfts.push(emptyNftRecord(nftId));
          }
        } catch {
          displayableNfts.push(emptyNftRecord(nftId));
        }
      }
      setNftDetailsList(displayableNfts);

      setLoadingNfts(false);
    };

    fetchNftDetails();
  }, [nftIds]);

  useEffect(() => {
    const fetchOptionDetails = async () => {
      setLoadingOptions(true);

      const displayableOptions: OptionRecord[] = [];
      for (const optionId of optionIds) {
        try {
          const optionRecordResponse = await commands.getOption({
            option_id: optionId,
          });
          if (optionRecordResponse.option) {
            displayableOptions.push(optionRecordResponse.option);
          }
        } catch {
          // Do nothing
        }
      }
      setOptionDetailsList(displayableOptions);

      setLoadingOptions(false);
    };

    fetchOptionDetails();
  }, [optionIds]);

  return (
    <div className='space-y-2'>
      {assets.tokens.length > 0 && (
        <div>
          <h4 className='font-semibold'>
            <Trans>Tokens</Trans>
          </h4>
          <ScrollArea className='max-h-32'>
            {loadingTokens ? (
              <p className='text-sm text-muted-foreground'>
                <Trans>Loading token details...</Trans>
              </p>
            ) : (
              <ul className='space-y-1'>
                {tokensWithNames.map((token: TokenWithName) => (
                  <li
                    key={token.asset_id}
                    className='text-sm flex items-center gap-2'
                  >
                    <AssetIcon
                      asset={{
                        icon_url: token.iconUrl ?? null,
                        kind: 'token',
                        revocation_address: null,
                        // TODO: Use Asset here and use the actual revocation address
                      }}
                      size='sm'
                    />
                    <div className='flex flex-col leading-tight'>
                      <span>
                        <NumberFormat
                          value={token.amount || '0'}
                          minimumFractionDigits={0}
                          maximumFractionDigits={token.precision}
                        />{' '}
                        {token.asset_id
                          ? token.displayName ||
                            `${token.asset_id.slice(0, 8)}...`
                          : t`Chia`}
                      </span>
                      {token.fee_policy && (
                        <span className='text-xs text-muted-foreground'>
                          <Trans>
                            Fee policy ({token.fee_policy.fee_basis_points} bps)
                          </Trans>
                        </span>
                      )}
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </ScrollArea>
        </div>
      )}

      {assets.nfts.filter((id) => id && typeof id === 'string').length > 0 && (
        <div>
          <h4 className='font-semibold'>NFTs</h4>
          <ScrollArea className='max-h-40'>
            {loadingNfts ? (
              <p className='text-sm text-muted-foreground'>
                <Trans>Loading NFT details...</Trans>
              </p>
            ) : nftDetailsList.length > 0 ? (
              <div className='grid grid-cols-4 gap-x-0 gap-y-1'>
                {nftDetailsList.map((nft) => {
                  return (
                    <div
                      key={nft.launcher_id}
                      className='flex flex-col items-center text-center'
                      title={`${nft.name}\nID: ${nft.launcher_id}`}
                    >
                      <AssetIcon
                        asset={{
                          icon_url: nft.icon_url ?? null,
                          kind: 'nft',
                          revocation_address: null,
                          // TODO: Use Asset here and use the actual revocation address
                        }}
                        size='sm'
                      />
                      {!nft.icon_url && (
                        <span className='text-xs truncate w-full'>
                          {nft.name}
                        </span>
                      )}
                    </div>
                  );
                })}
              </div>
            ) : (
              <p className='text-sm text-muted-foreground'>
                {type === 'offered' ? (
                  <Trans>No NFTs offered or details unavailable.</Trans>
                ) : (
                  <Trans>No NFTs requested or details unavailable.</Trans>
                )}
              </p>
            )}
          </ScrollArea>
        </div>
      )}

      {assets.options.filter((id) => id && typeof id === 'string').length >
        0 && (
        <div>
          <h4 className='font-semibold'>Options</h4>
          <ScrollArea className='max-h-40'>
            {loadingOptions ? (
              <p className='text-sm text-muted-foreground'>
                <Trans>Loading option details...</Trans>
              </p>
            ) : (
              <div className='grid grid-cols-4 gap-x-0 gap-y-1'>
                {optionDetailsList.length > 0
                  ? // Show options with details when available
                    optionDetailsList.map((option) => {
                      return (
                        <div
                          key={option.launcher_id}
                          className='flex flex-col items-center text-center'
                          title={`${option.name}\nID: ${option.launcher_id}`}
                        >
                          <AssetIcon
                            asset={{
                              icon_url: null,
                              kind: 'option',
                              revocation_address: null,
                              // TODO: Use Asset here and use the actual revocation address
                            }}
                            size='sm'
                          />
                          <span className='text-xs truncate w-full'>
                            {option.name}
                          </span>
                        </div>
                      );
                    })
                  : // Show option IDs when details are not available
                    optionIds.map((optionId) => {
                      return (
                        <div
                          key={optionId}
                          className='flex flex-col items-center text-center'
                          title={`Option ID: ${optionId}`}
                        >
                          <AssetIcon
                            asset={{
                              icon_url: null,
                              kind: 'option',
                              revocation_address: null,
                            }}
                            size='sm'
                          />
                          <span className='text-xs truncate w-full'>
                            {optionId.slice(6, 15)}...
                          </span>
                        </div>
                      );
                    })}
              </div>
            )}
          </ScrollArea>
        </div>
      )}

      {assets.tokens.length === 0 &&
        assets.nfts.filter((n) => n).length === 0 &&
        assets.options.filter((o) => o).length === 0 && (
          <p className='text-sm text-muted-foreground'>
            {type === 'offered' ? (
              <Trans>Nothing offered.</Trans>
            ) : (
              <Trans>Nothing requested.</Trans>
            )}
          </p>
        )}
    </div>
  );
}

export function MakeOfferConfirmationDialog({
  open,
  onOpenChange,
  onConfirm,
  offerState,
  splitNftOffers,
  fee,
  enabledMarketplaces,
  setEnabledMarketplaces,
}: MakeOfferConfirmationDialogProps) {
  const [xchToken, setXchToken] = useState<TokenRecord | null>(null);

  useEffect(() => {
    const fetchXchToken = async () => {
      try {
        const tokenResponse = await commands.getToken({ asset_id: null });
        setXchToken(tokenResponse.token);
      } catch (error) {
        console.error('Error fetching XCH token info:', error);
        setXchToken(null);
      }
    };

    if (open) {
      fetchXchToken();
    }
  }, [open]);

  const handleConfirm = () => {
    onConfirm();
    onOpenChange(false);
  };

  const numOfferedNfts = offerState.offered.nfts.filter(
    (nft: string) => !!nft,
  ).length;

  const isSplitting = splitNftOffers && numOfferedNfts > 1;
  const numberOfOffers = isSplitting ? numOfferedNfts : 1;

  const feePerOffer = fee || '0';
  const totalFee = isSplitting
    ? new BigNumber(feePerOffer).multipliedBy(numOfferedNfts).toString()
    : feePerOffer;
  const hasFee = new BigNumber(feePerOffer).gt(0);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className='sm:max-w-lg'>
        <DialogHeader>
          <DialogTitle>
            <Trans>Confirm Offer Creation</Trans>
          </DialogTitle>
          <DialogDescription>
            <Trans>
              Please review the details of your offer(s) before proceeding.
            </Trans>
          </DialogDescription>
        </DialogHeader>

        <div className='space-y-4 py-4'>
          <div>
            <h3 className='text-md font-semibold mb-1'>
              <Trans>Summary</Trans>
            </h3>
            {isSplitting ? (
              <p className='text-sm'>
                <Trans>
                  You are about to create{' '}
                  <span className='font-bold'>{numberOfOffers}</span> individual
                  offers. Each offer will request the same assets and offer one
                  of the selected NFTs.
                </Trans>
              </p>
            ) : (
              <p className='text-sm'>
                <Trans>
                  You are about to create <span className='font-bold'>1</span>{' '}
                  offer.
                </Trans>
              </p>
            )}
          </div>

          <div className='grid grid-cols-2 gap-4'>
            <div>
              <h3 className='text-md font-semibold mb-2'>
                <Trans>You Are Offering</Trans>
              </h3>
              <AssetDisplay assets={offerState.offered} type='offered' />
            </div>
            <div>
              <h3 className='text-md font-semibold mb-2'>
                <Trans>You Are Requesting</Trans>
                {isSplitting && (
                  <span className='text-xs text-muted-foreground ml-1'>
                    <Trans>(for each offer)</Trans>
                  </span>
                )}
              </h3>

              {(() => {
                const hasRequestedTokens = offerState.requested.tokens.some(
                  (token) => new BigNumber(token.amount || '0').gt(0),
                );
                const hasRequestedNfts =
                  offerState.requested.nfts.filter((n) => n).length > 0;
                const hasRequestedOptions =
                  offerState.requested.options.filter((o) => o).length > 0;
                if (
                  !hasRequestedTokens &&
                  !hasRequestedNfts &&
                  !hasRequestedOptions
                ) {
                  return (
                    <div className='flex items-center gap-2 p-3 mb-2 rounded border border-yellow-400 bg-yellow-50 text-yellow-900 dark:bg-yellow-900/20 dark:text-yellow-200'>
                      <AlertTriangle className='h-5 w-5 text-yellow-500 flex-shrink-0' />
                      <span className='font-semibold'>
                        <Trans>
                          Warning: This is a one-sided offer. You are not
                          requesting anything in return.
                        </Trans>
                      </span>
                    </div>
                  );
                }
                return null;
              })()}
              <AssetDisplay assets={offerState.requested} type='requested' />
            </div>
          </div>

          {(hasFee || isSplitting) && (
            <div>
              <h3 className='text-md font-semibold mb-1'>
                <Trans>Fees</Trans>
              </h3>
              {isSplitting ? (
                <>
                  <p className='text-sm'>
                    <Trans>Fee per offer:</Trans>{' '}
                    <NumberFormat
                      value={feePerOffer}
                      minimumFractionDigits={2}
                      maximumFractionDigits={xchToken?.precision ?? 12}
                    />{' '}
                    {xchToken
                      ? getAssetDisplayName(
                          xchToken.name,
                          xchToken.ticker,
                          'token',
                        )
                      : 'XCH'}
                  </p>
                  <p className='text-sm'>
                    <Trans>Total fees for {numberOfOffers} offers:</Trans>{' '}
                    <NumberFormat
                      value={totalFee}
                      minimumFractionDigits={2}
                      maximumFractionDigits={xchToken?.precision ?? 12}
                    />{' '}
                    {xchToken
                      ? getAssetDisplayName(
                          xchToken.name,
                          xchToken.ticker,
                          'token',
                        )
                      : 'XCH'}
                  </p>
                </>
              ) : (
                <p className='text-sm'>
                  <Trans>Network Fee:</Trans>{' '}
                  <NumberFormat
                    value={feePerOffer}
                    minimumFractionDigits={2}
                    maximumFractionDigits={xchToken?.precision ?? 12}
                  />{' '}
                  {xchToken
                    ? getAssetDisplayName(
                        xchToken.name,
                        xchToken.ticker,
                        'token',
                      )
                    : 'XCH'}
                </p>
              )}
              {(fee || '0') === '0' && (
                <p className='text-xs text-muted-foreground'>
                  <Trans>
                    (Plus any blockchain royalties for offered assets)
                  </Trans>
                </p>
              )}
            </div>
          )}

          {enabledMarketplaces && (
            <div className='flex flex-col gap-4 pt-2'>
              {marketplaces.map((marketplace) => {
                const isSupported = marketplace.isSupported(
                  offerState,
                  isSplitting,
                );
                if (!isSupported) return null;

                return (
                  <div
                    key={marketplace.id}
                    className='flex items-center space-x-2'
                  >
                    <Switch
                      id={`auto-upload-${marketplace.id}`}
                      checked={enabledMarketplaces[marketplace.id] || false}
                      onCheckedChange={(checked) =>
                        setEnabledMarketplaces?.({
                          ...enabledMarketplaces,
                          [marketplace.id]: checked,
                        })
                      }
                    />
                    <Label
                      htmlFor={`auto-upload-${marketplace.id}`}
                      className='flex flex-col'
                    >
                      <span>
                        <Trans>Upload to {marketplace.name}</Trans>
                      </span>
                      {enabledMarketplaces[marketplace.id] && (
                        <span className='text-xs text-muted-foreground'>
                          <Trans>
                            This will make your offer(s) immediately public and
                            takeable on {marketplace.name}.
                          </Trans>
                        </span>
                      )}
                    </Label>
                  </div>
                );
              })}
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant='outline' onClick={() => onOpenChange(false)}>
            <Trans>Cancel</Trans>
          </Button>
          <Button onClick={handleConfirm}>
            <Trans>Confirm & Create</Trans>
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
