import {
  Assets,
  CatAmount,
  commands,
  NftRecord,
  TokenRecord,
} from '@/bindings';
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
import { OfferState } from '@/state';
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
  enabledMarketplaces: Record<string, boolean>;
  setEnabledMarketplaces: (marketplaces: Record<string, boolean>) => void;
}
interface CatWithName extends CatAmount {
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
  const xchAmount = assets.xch || '0';
  const hasXch = new BigNumber(xchAmount).gt(0);
  const [nftDetailsList, setNftDetailsList] = useState<NftRecord[]>([]);
  const [loadingNfts, setLoadingNfts] = useState(false);
  const [catsWithNames, setCatsWithNames] = useState<CatWithName[]>([]);
  const [loadingCats, setLoadingCats] = useState(false);
  const [xchToken, setXchToken] = useState<TokenRecord | null>(null);
  const [loadingXch, setLoadingXch] = useState(false);

  // Create a stable reference for NFT IDs to prevent infinite re-renders
  const nftIds = useMemo(() => {
    if (!assets.nfts || assets.nfts.length === 0) return [];
    return assets.nfts.filter((id) => id && typeof id === 'string');
  }, [assets.nfts]);

  useEffect(() => {
    const fetchXchToken = async () => {
      if (!hasXch) {
        setXchToken(null);
        return;
      }
      setLoadingXch(true);

      try {
        const tokenResponse = await commands.getToken({ asset_id: null });
        setXchToken(tokenResponse.token);
      } catch (error) {
        console.error('Error fetching XCH token info:', error);
        setXchToken(null);
      }
      setLoadingXch(false);
    };

    fetchXchToken();
  }, [hasXch]);

  useEffect(() => {
    const fetchCatNames = async () => {
      if (assets.cats.length === 0) {
        setCatsWithNames([]);
        return;
      }
      setLoadingCats(true);

      try {
        const catsWithNamesPromises = assets.cats.map(async (cat) => {
          try {
            const tokenResponse = await commands.getToken({
              asset_id: cat.asset_id,
            });
            const token = tokenResponse.token;
            if (token) {
              return {
                ...cat,
                displayName: getAssetDisplayName(
                  token.name,
                  token.ticker,
                  'token',
                ),
                iconUrl: token.icon_url,
                precision: token.precision,
              };
            } else {
              return {
                ...cat,
                displayName: getAssetDisplayName(null, null, 'token'),
                iconUrl: null,
                precision: 3,
              };
            }
          } catch (error) {
            console.error(
              `Failed to fetch token info for ${cat.asset_id}:`,
              error,
            );
            return {
              ...cat,
              displayName: getAssetDisplayName(null, null, 'token'),
              iconUrl: null,
              precision: 3,
            };
          }
        });

        const catsWithNamesResults = await Promise.all(catsWithNamesPromises);
        setCatsWithNames(catsWithNamesResults);
      } catch (error) {
        console.error('Error fetching CAT names:', error);
        // Fallback to original cats without names
        setCatsWithNames(
          assets.cats.map((cat) => ({
            ...cat,
            displayName: getAssetDisplayName(null, null, 'token'),
            iconUrl: null,
          })),
        );
      }
      setLoadingCats(false);
    };

    fetchCatNames();
  }, [assets.cats]);

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

  return (
    <div className='space-y-2'>
      {hasXch && (
        <div>
          <h4 className='font-semibold'>{xchToken?.ticker ?? 'XCH'}</h4>
          {loadingXch ? (
            <p className='text-sm text-muted-foreground'>
              <Trans>Loading {xchToken?.ticker ?? 'XCH'} details...</Trans>
            </p>
          ) : (
            <div className='flex items-center gap-2'>
              <AssetIcon
                asset={{
                  icon_url: xchToken?.icon_url ?? null,
                  kind: 'token',
                  revocation_address: xchToken?.revocation_address ?? null,
                }}
                size='sm'
              />
              <span>
                <NumberFormat
                  value={xchAmount}
                  minimumFractionDigits={0}
                  maximumFractionDigits={xchToken?.precision ?? 12}
                />{' '}
                {xchToken
                  ? getAssetDisplayName(xchToken.name, xchToken.ticker, 'token')
                  : 'XCH'}
              </span>
            </div>
          )}
        </div>
      )}
      {assets.cats.length > 0 && (
        <div>
          <h4 className='font-semibold'>
            <Trans>Tokens</Trans>
          </h4>
          <ScrollArea className='max-h-32'>
            {loadingCats ? (
              <p className='text-sm text-muted-foreground'>
                <Trans>Loading token details...</Trans>
              </p>
            ) : (
              <ul className='space-y-1'>
                {catsWithNames.map((cat: CatWithName) => (
                  <li
                    key={cat.asset_id}
                    className='text-sm flex items-center gap-2'
                  >
                    <AssetIcon
                      asset={{
                        icon_url: cat.iconUrl ?? null,
                        kind: 'token',
                        revocation_address: null,
                        // TODO: Use Asset here and use the actual revocation address
                      }}
                      size='sm'
                    />
                    <span>
                      <NumberFormat
                        value={cat.amount || '0'}
                        minimumFractionDigits={0}
                        maximumFractionDigits={cat.precision}
                      />{' '}
                      {cat.displayName || `${cat.asset_id.slice(0, 8)}...`}
                    </span>
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
      {!hasXch &&
        assets.cats.length === 0 &&
        assets.nfts.filter((n) => n).length === 0 && (
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
              {/* One-sided offer warning */}
              {(() => {
                const hasRequestedXch = new BigNumber(
                  offerState.requested.xch || '0',
                ).gt(0);
                const hasRequestedCats = offerState.requested.cats.some((cat) =>
                  new BigNumber(cat.amount || '0').gt(0),
                );
                const hasRequestedNfts =
                  offerState.requested.nfts.filter((n) => n).length > 0;
                if (
                  !hasRequestedXch &&
                  !hasRequestedCats &&
                  !hasRequestedNfts
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
                      setEnabledMarketplaces({
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
