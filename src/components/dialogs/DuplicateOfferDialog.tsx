import { commands, NetworkKind, OfferAmount, OfferRecord } from '@/bindings';
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
import { useErrors } from '@/hooks/useErrors';
import { useBiometric } from '@/hooks/useBiometric';
import { marketplaces } from '@/lib/marketplaces';
import { fromMojos, getAssetDisplayName } from '@/lib/utils';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import BigNumber from 'bignumber.js';
import { LoaderCircleIcon, Minus, Plus } from 'lucide-react';
import { useEffect, useState } from 'react';

const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

interface DuplicateOfferDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  record: OfferRecord;
  onDone: () => void;
}

type Phase = 'config' | 'progress';

export function DuplicateOfferDialog({
  open,
  onOpenChange,
  record,
  onDone,
}: DuplicateOfferDialogProps) {
  const walletState = useWalletState();
  const { addError } = useErrors();
  const { promptIfEnabled } = useBiometric();

  const [copies, setCopies] = useState(1);
  const [phase, setPhase] = useState<Phase>('config');
  const [enabledMarketplaces, setEnabledMarketplaces] = useState<
    Record<string, boolean>
  >({});
  const [balanceError, setBalanceError] = useState<string | null>(null);
  const [network, setNetwork] = useState<NetworkKind | null>(null);
  const [isCreating, setIsCreating] = useState(false);
  const [isUploading, setIsUploading] = useState(false);
  const [currentStep, setCurrentStep] = useState<'creating' | 'uploading'>(
    'creating',
  );
  const [currentOfferIndex, setCurrentOfferIndex] = useState(0);
  const [currentMarketplaceIndex, setCurrentMarketplaceIndex] = useState(0);
  const [isDone, setIsDone] = useState(false);

  // Reset state when dialog closes
  useEffect(() => {
    if (!open) {
      setCopies(1);
      setPhase('config');
      setIsCreating(false);
      setIsUploading(false);
      setIsDone(false);
      setCurrentOfferIndex(0);
      setCurrentMarketplaceIndex(0);
      setCurrentStep('creating');
      setBalanceError(null);
      setEnabledMarketplaces({});
    }
  }, [open]);

  // Fetch network once
  useEffect(() => {
    if (open) {
      commands.getNetwork({}).then((data) => setNetwork(data.kind));
    }
  }, [open]);

  useEffect(() => {
    if (!open) return;

    const validate = async () => {
      // XCH check: (fee + any XCH offered) × copies ≤ selectable_balance (mojos)
      const xchAsset = record.summary.maker.find(
        (a) => a.asset.asset_id === null,
      );
      const feeMojos = BigNumber(record.summary.fee.toString());
      const xchOfferedMojos = xchAsset
        ? BigNumber(xchAsset.amount.toString())
        : BigNumber(0);
      const totalXchNeeded = feeMojos
        .plus(xchOfferedMojos)
        .multipliedBy(copies);
      const xchBalance = BigNumber(walletState.sync.selectable_balance);

      if (totalXchNeeded.gt(xchBalance)) {
        const needed = fromMojos(totalXchNeeded.toString(), 12).toString();
        const have = fromMojos(xchBalance.toString(), 12).toString();
        setBalanceError(
          t`Insufficient XCH balance: need ${needed} XCH for ${copies} copies, have ${have}`,
        );
        return;
      }

      // Per-CAT check
      for (const asset of record.summary.maker.filter(
        (a) => a.asset.kind === 'token' && a.asset.asset_id !== null,
      )) {
        const neededMojos = BigNumber(asset.amount.toString()).multipliedBy(
          copies,
        );
        try {
          const resp = await commands.getToken({
            asset_id: asset.asset.asset_id,
          });
          if (resp.token) {
            const haveMojos = BigNumber(resp.token.selectable_balance);
            if (neededMojos.gt(haveMojos)) {
              const displayName = getAssetDisplayName(
                resp.token.name,
                resp.token.ticker,
                'token',
              );
              const needed = fromMojos(
                neededMojos.toString(),
                resp.token.precision,
              ).toString();
              const have = fromMojos(
                haveMojos.toString(),
                resp.token.precision,
              ).toString();
              setBalanceError(
                t`Insufficient balance: need ${needed} ${displayName} for ${copies} copies, have ${have}`,
              );
              return;
            }
          }
        } catch {
          // skip check if fetch fails
        }
      }

      setBalanceError(null);
    };

    validate();
  }, [
    copies,
    open,
    record.summary.fee,
    record.summary.maker,
    walletState.sync.selectable_balance,
  ]);

  const handleDuplicate = async () => {
    // Check biometric before transitioning to progress phase
    if (!(await promptIfEnabled())) {
      return;
    }

    setPhase('progress');
    setIsCreating(true);
    setCurrentStep('creating');

    const offeredAssets: OfferAmount[] = record.summary.maker.map((a) => ({
      asset_id: a.asset.asset_id,
      amount: a.amount,
    }));
    const requestedAssets: OfferAmount[] = record.summary.taker.map((a) => ({
      asset_id: a.asset.asset_id,
      amount: a.amount,
    }));
    const fee = record.summary.fee;
    const expiresAtSecond = record.summary.expiration_timestamp ?? null;

    const createdOffers: string[] = [];

    try {
      for (let i = 0; i < copies; i++) {
        setCurrentOfferIndex(i);
        const data = await commands.makeOffer({
          offered_assets: offeredAssets,
          requested_assets: requestedAssets,
          fee,
          expires_at_second: expiresAtSecond,
        });
        createdOffers.push(data.offer);
      }
    } catch (error) {
      addError({
        kind: 'invalid',
        reason:
          error instanceof Error
            ? t`Failed on copy ${createdOffers.length + 1} of ${copies}: ${error.message}`
            : t`Failed to create offer`,
      });
      onOpenChange(false);
      return;
    }

    setIsCreating(false);

    // Upload to marketplaces
    const enabledConfigs = marketplaces.filter(
      (m) => enabledMarketplaces[m.id],
    );

    if (enabledConfigs.length > 0 && network) {
      setIsUploading(true);
      setCurrentStep('uploading');

      for (const [mi, marketplace] of enabledConfigs.entries()) {
        setCurrentMarketplaceIndex(mi);
        for (const [oi, offer] of createdOffers.entries()) {
          setCurrentOfferIndex(oi);
          try {
            await marketplace.uploadToMarketplace(offer, network === 'testnet');
            if (oi < createdOffers.length - 1) {
              await delay(500);
            }
          } catch (error) {
            addError({
              kind: 'upload',
              reason: t`Failed to upload offer ${oi + 1} to ${marketplace.name}: ${error instanceof Error ? error.message : String(error)}`,
            });
            // Break inner loop only — consistent with OfferCreationProgressDialog behavior.
            // Typically if one offer fails, the rest will too.
            break;
          }
        }
      }

      setIsUploading(false);
    }

    setIsDone(true);
  };

  const isInProgress = isCreating || isUploading;
  const supportedMarketplaces = marketplaces.filter((m) =>
    m.isSupported(record.summary, false),
  );

  const getProgressMessage = () => {
    if (currentStep === 'creating') {
      return (
        <Trans>
          Creating offer {currentOfferIndex + 1} of {copies}...
        </Trans>
      );
    }
    const currentMarketplace = supportedMarketplaces.filter(
      (m) => enabledMarketplaces[m.id],
    )[currentMarketplaceIndex];
    return (
      <Trans>
        Uploading offer {currentOfferIndex + 1} of {copies} to{' '}
        {currentMarketplace?.name}...
      </Trans>
    );
  };

  return (
    <Dialog
      open={open}
      onOpenChange={(isOpen) => {
        if (!isInProgress) onOpenChange(isOpen);
      }}
    >
      <DialogContent
        className='sm:max-w-md'
        onInteractOutside={(e) => {
          if (isInProgress) e.preventDefault();
        }}
        onEscapeKeyDown={(e) => {
          if (isInProgress) e.preventDefault();
        }}
      >
        {phase === 'config' ? (
          <>
            <DialogHeader>
              <DialogTitle>
                <Trans>Duplicate Offer</Trans>
              </DialogTitle>
              <DialogDescription>
                <Trans>
                  Create identical copies of this offer. All copies will use the
                  same assets and amounts as the original. Timestamp-based
                  expiry is preserved; block-height expiry is not supported and
                  will be omitted.
                </Trans>
              </DialogDescription>
            </DialogHeader>

            <div className='space-y-4 py-4'>
              <div>
                <h3 className='text-md font-semibold mb-1'>
                  <Trans>Number of Copies</Trans>
                </h3>
                <div className='flex items-center gap-2'>
                  <Button
                    variant='outline'
                    size='icon'
                    className='h-8 w-8'
                    onClick={() => setCopies(Math.max(1, copies - 1))}
                    disabled={copies <= 1}
                    type='button'
                  >
                    <Minus className='h-4 w-4' />
                  </Button>
                  <span className='w-8 text-center font-mono'>{copies}</span>
                  <Button
                    variant='outline'
                    size='icon'
                    className='h-8 w-8'
                    onClick={() => setCopies(Math.min(10, copies + 1))}
                    disabled={copies >= 10}
                    type='button'
                  >
                    <Plus className='h-4 w-4' />
                  </Button>
                </div>
                {balanceError && (
                  <p className='text-sm text-destructive mt-1'>
                    {balanceError}
                  </p>
                )}
              </div>

              {supportedMarketplaces.length > 0 && (
                <div className='flex flex-col gap-4 pt-2'>
                  {supportedMarketplaces.map((marketplace) => (
                    <div
                      key={marketplace.id}
                      className='flex items-center space-x-2'
                    >
                      <Switch
                        id={`dup-upload-${marketplace.id}`}
                        checked={enabledMarketplaces[marketplace.id] || false}
                        onCheckedChange={(checked) =>
                          setEnabledMarketplaces({
                            ...enabledMarketplaces,
                            [marketplace.id]: checked,
                          })
                        }
                      />
                      <Label
                        htmlFor={`dup-upload-${marketplace.id}`}
                        className='flex flex-col'
                      >
                        <span>
                          <Trans>Upload to {marketplace.name}</Trans>
                        </span>
                        {enabledMarketplaces[marketplace.id] && (
                          <span className='text-xs text-muted-foreground'>
                            <Trans>
                              This will make your offer(s) immediately public
                              and takeable on {marketplace.name}.
                            </Trans>
                          </span>
                        )}
                      </Label>
                    </div>
                  ))}
                </div>
              )}
            </div>

            <DialogFooter>
              <Button variant='outline' onClick={() => onOpenChange(false)}>
                <Trans>Cancel</Trans>
              </Button>
              <Button onClick={handleDuplicate} disabled={!!balanceError}>
                <Trans>Duplicate</Trans>
              </Button>
            </DialogFooter>
          </>
        ) : (
          <>
            <DialogHeader>
              <DialogTitle>
                {isInProgress ? (
                  <div className='flex items-center gap-2'>
                    <LoaderCircleIcon
                      className='h-4 w-4 animate-spin'
                      aria-hidden='true'
                    />
                    {currentStep === 'creating' ? (
                      <Trans>Creating Offers</Trans>
                    ) : (
                      <Trans>Uploading Offers</Trans>
                    )}
                  </div>
                ) : (
                  <Trans>Offers Created</Trans>
                )}
              </DialogTitle>
              <DialogDescription>
                {isInProgress ? (
                  <div className='space-y-2'>
                    <p>
                      <Trans>
                        Please wait while your offers are being{' '}
                        {currentStep === 'creating' ? 'created' : 'uploaded'}
                        ...
                      </Trans>
                    </p>
                    <p className='text-sm text-muted-foreground'>
                      {getProgressMessage()}
                    </p>
                  </div>
                ) : (
                  <Trans>
                    {copies} offer{copies > 1 ? 's have' : ' has'} been created
                    successfully
                    {Object.values(enabledMarketplaces).some(Boolean)
                      ? ' and uploaded to the selected marketplaces'
                      : ''}
                    .
                  </Trans>
                )}
              </DialogDescription>
            </DialogHeader>
            <DialogFooter>
              {isDone && (
                <Button
                  onClick={() => {
                    onOpenChange(false);
                    onDone();
                  }}
                >
                  <Trans>Done</Trans>
                </Button>
              )}
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
}
