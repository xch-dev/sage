import { AssetIcon } from '@/components/AssetIcon';
import ConfirmationDialog from '@/components/ConfirmationDialog';
import { DidConfirmation } from '@/components/confirmations/DidConfirmation';
import { FeeOnlyDialog } from '@/components/FeeOnlyDialog';
import { TransferDialog } from '@/components/TransferDialog';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Skeleton } from '@/components/ui/skeleton';
import { useErrors } from '@/hooks/useErrors';
import { getMintGardenProfile } from '@/lib/marketplaces';
import { getAssetDisplayName, toMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { openUrl } from '@tauri-apps/plugin-opener';
import {
  ActivityIcon,
  Copy,
  ExternalLinkIcon,
  EyeIcon,
  EyeOff,
  Flame,
  MoreVerticalIcon,
  PenIcon,
  SendIcon,
} from 'lucide-react';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { toast } from 'react-toastify';
import {
  AssetKind,
  commands,
  DidRecord,
  TransactionResponse,
} from '../bindings';
import { CustomError } from '../contexts/ErrorContext';

export interface MintGardenProfile {
  encoded_id: string;
  name: string;
  avatar_uri: string | null;
  is_unknown: boolean;
}

export interface ProfileCardProps {
  // For MintGarden profiles (DID string only)
  did: DidRecord | string;
  variant?: 'default' | 'compact' | 'card';
  className?: string;
  updateDids?: () => void;
  allowMintGardenProfile?: boolean;
}

export function ProfileCard({
  did,
  variant = 'default',
  className = '',
  allowMintGardenProfile: allowMintGardenProfile = true,
  updateDids,
}: ProfileCardProps) {
  const { addError } = useErrors();
  const walletState = useWalletState();
  const navigate = useNavigate();

  // this component can be used to display a DID record or a DID string
  // if it is a DID DidRecord, it will be treated as a local DID and will have
  // additional actions available
  const isOwned = typeof did !== 'string';
  const didRecord: DidRecord =
    typeof did === 'string'
      ? {
          launcher_id: did,
          name: `${did.slice(9, 19)}...${did.slice(-4)}`,
          visible: true,
          created_height: null,
          recovery_hash: null,
          coin_id: '0',
          address: '',
          amount: 0,
        }
      : did;

  const [mintGardenProfile, setMintGardenProfile] = useState<MintGardenProfile>(
    {
      encoded_id: didRecord.launcher_id,
      name: didRecord.name ?? '',
      avatar_uri: null,
      is_unknown: true,
    },
  );

  const didAsset = {
    icon_url: mintGardenProfile.avatar_uri,
    kind: 'did' as AssetKind,
    revocation_address: null,
    name: !mintGardenProfile.is_unknown
      ? mintGardenProfile.name
      : getAssetDisplayName(
          didRecord?.name || mintGardenProfile?.name,
          null,
          'did',
        ),
    ticker: '',
    precision: 0,
    asset_id: didRecord.launcher_id,
    balance: '0',
    balanceInUsd: '0',
    priceInUsd: '0',
  };

  const [isMintGardenLoading, setIsMintGardenLoading] = useState(false);

  // State for local DID actions (only when did is owned)
  const [name, setName] = useState('');
  const [response, setResponse] = useState<TransactionResponse | null>(null);
  const [renameOpen, setRenameOpen] = useState(false);
  const [transferOpen, setTransferOpen] = useState(false);
  const [burnOpen, setBurnOpen] = useState(false);
  const [normalizeOpen, setNormalizeOpen] = useState(false);
  const [isTransferring, setIsTransferring] = useState(false);
  const [isBurning, setIsBurning] = useState(false);
  const [isNormalizing, setIsNormalizing] = useState(false);
  const [transferAddress, setTransferAddress] = useState('');

  // Fetch MintGarden profile data
  useEffect(() => {
    if (!didRecord.launcher_id) return;

    if (allowMintGardenProfile) {
      setIsMintGardenLoading(true);
      getMintGardenProfile(didRecord.launcher_id)
        .then((profileData) => {
          setMintGardenProfile(profileData);
        })
        .catch(() => {
          // Create fallback profile for failed lookups
          setMintGardenProfile({
            encoded_id: didRecord.launcher_id,
            name: didRecord.name ?? '',
            avatar_uri: null,
            is_unknown: true,
          });
        })
        .finally(() => {
          setIsMintGardenLoading(false);
        });
    } else {
      setMintGardenProfile({
        encoded_id: didRecord.launcher_id,
        name: didRecord.name ?? '',
        avatar_uri: null,
        is_unknown: true,
      });
    }
  }, [didRecord.launcher_id, didRecord.name, allowMintGardenProfile]);

  // Owned DID action handlers
  const rename = () => {
    if (!name || !isOwned) return;

    commands
      .updateDid({
        did_id: didRecord.launcher_id,
        name,
        visible: didRecord.visible,
      })
      .then(() => updateDids?.())
      .catch((err) => addError(err as CustomError))
      .finally(() => {
        setRenameOpen(false);
        setName('');
      });
  };

  const toggleVisibility = () => {
    if (!isOwned) return;

    commands
      .updateDid({
        did_id: didRecord.launcher_id,
        name: didRecord.name,
        visible: !didRecord.visible,
      })
      .then(() => updateDids?.())
      .catch((err) => addError(err as CustomError));
  };

  const onTransferSubmit = (address: string, fee: string) => {
    if (!isOwned) return;

    setIsTransferring(true);
    setTransferAddress(address);
    commands
      .transferDids({
        did_ids: [didRecord.launcher_id],
        address,
        fee: toMojos(fee, walletState.sync.unit.decimals),
      })
      .then(setResponse)
      .catch((err) => {
        setIsTransferring(false);
        addError(err as CustomError);
      })
      .finally(() => setTransferOpen(false));
  };

  const onBurnSubmit = (fee: string) => {
    if (!isOwned) return;

    setIsBurning(true);
    commands
      .transferDids({
        did_ids: [didRecord.launcher_id],
        address: walletState.sync.burn_address,
        fee: toMojos(fee, walletState.sync.unit.decimals),
      })
      .then(setResponse)
      .catch((err) => {
        setIsBurning(false);
        addError(err as CustomError);
      })
      .finally(() => setBurnOpen(false));
  };

  const onNormalizeSubmit = (fee: string) => {
    if (!isOwned) return;

    setIsNormalizing(true);
    commands
      .normalizeDids({
        did_ids: [didRecord.launcher_id],
        fee: toMojos(fee, walletState.sync.unit.decimals),
      })
      .then(setResponse)
      .catch((err) => {
        setIsNormalizing(false);
        addError(err as CustomError);
      })
      .finally(() => setNormalizeOpen(false));
  };

  const handleMintGardenClick = () => {
    if (!mintGardenProfile.is_unknown) {
      openUrl(`https://mintgarden.io/${mintGardenProfile.encoded_id}`);
    }
  };

  const handleDidClick = () => {
    navigate(`/dids/${didRecord.launcher_id}`);
  };

  // Loading state
  if (isMintGardenLoading) {
    return (
      <div className={`flex items-center gap-2 ${className}`}>
        <Skeleton className='w-8 h-8 rounded-full' />
        <div className='space-y-1'>
          <Skeleton className='h-4 w-24' />
        </div>
      </div>
    );
  }

  if (variant === 'compact') {
    return (
      <div className={`flex items-center gap-2 ${className}`}>
        <AssetIcon asset={didAsset} size='sm' />
        <span className='text-sm font-medium truncate'>{didAsset.name}</span>

        {!mintGardenProfile.is_unknown && (
          <Button
            variant='ghost'
            size='sm'
            onClick={handleMintGardenClick}
            className='p-1 h-auto'
            title={`View ${didAsset.name} on MintGarden`}
          >
            <ExternalLinkIcon className='w-3 h-3' />
          </Button>
        )}
      </div>
    );
  }

  if (variant === 'card') {
    return (
      <>
        <Card
          className={`${className} ${
            isOwned && !didRecord.visible
              ? 'opacity-50 grayscale'
              : isOwned && didRecord.created_height === null
                ? 'pulsate-opacity'
                : ''
          }`}
        >
          <CardHeader className='-mt-2 flex flex-row items-center justify-between space-y-0 pb-2 pr-2 space-x-2'>
            <CardTitle className='text-md font-medium truncate flex items-center'>
              <AssetIcon asset={didAsset} size='md' className='mr-2' />
              {mintGardenProfile.is_unknown
                ? didAsset.name
                : mintGardenProfile.name}{' '}
              {isOwned &&
                !mintGardenProfile.is_unknown &&
                didRecord.name &&
                didRecord.name !== mintGardenProfile.name && (
                  <span className='ml-2 text-sm text-gray-500 font-mono truncate'>
                    {`(${didRecord.name})`}
                  </span>
                )}
            </CardTitle>

            {isOwned ? (
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button variant='ghost' size='icon'>
                    <MoreVerticalIcon className='h-5 w-5' />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align='end'>
                  <DropdownMenuGroup>
                    {!mintGardenProfile.is_unknown && (
                      <>
                        <DropdownMenuItem
                          className='cursor-pointer'
                          onClick={(e) => {
                            e.stopPropagation();
                            handleMintGardenClick();
                          }}
                        >
                          <ExternalLinkIcon className='mr-2 h-4 w-4' />
                          <Trans>View on MintGarden</Trans>
                        </DropdownMenuItem>
                        <DropdownMenuSeparator />
                      </>
                    )}

                    <DropdownMenuItem
                      className='cursor-pointer'
                      onClick={(e) => {
                        e.stopPropagation();
                        setTransferOpen(true);
                      }}
                      disabled={didRecord.created_height === null}
                    >
                      <SendIcon className='mr-2 h-4 w-4' />
                      <Trans>Transfer</Trans>
                    </DropdownMenuItem>

                    {didRecord.recovery_hash === null && (
                      <DropdownMenuItem
                        className='cursor-pointer'
                        onClick={(e) => {
                          e.stopPropagation();
                          setNormalizeOpen(true);
                        }}
                        disabled={didRecord.created_height === null}
                      >
                        <ActivityIcon className='mr-2 h-4 w-4' />
                        <Trans>Normalize</Trans>
                      </DropdownMenuItem>
                    )}

                    <DropdownMenuItem
                      className='cursor-pointer'
                      onClick={(e) => {
                        e.stopPropagation();
                        setBurnOpen(true);
                      }}
                      disabled={didRecord.created_height === null}
                    >
                      <Flame className='mr-2 h-4 w-4' />
                      <Trans>Burn</Trans>
                    </DropdownMenuItem>

                    <DropdownMenuSeparator />

                    <DropdownMenuItem
                      className='cursor-pointer'
                      onClick={(e) => {
                        e.stopPropagation();
                        writeText(didRecord.launcher_id);
                        toast.success(t`DID ID copied to clipboard`);
                      }}
                    >
                      <Copy className='mr-2 h-4 w-4' />
                      <Trans>Copy ID</Trans>
                    </DropdownMenuItem>

                    <DropdownMenuItem
                      className='cursor-pointer'
                      onClick={(e) => {
                        e.stopPropagation();
                        setRenameOpen(true);
                      }}
                    >
                      <PenIcon className='mr-2 h-4 w-4' />
                      <Trans>Rename</Trans>
                    </DropdownMenuItem>

                    <DropdownMenuItem
                      className='cursor-pointer'
                      onClick={(e) => {
                        e.stopPropagation();
                        toggleVisibility();
                      }}
                    >
                      {didRecord.visible ? (
                        <EyeOff className='mr-2 h-4 w-4' />
                      ) : (
                        <EyeIcon className='mr-2 h-4 w-4' />
                      )}
                      <span>{didRecord.visible ? t`Hide` : t`Show`}</span>
                    </DropdownMenuItem>
                  </DropdownMenuGroup>
                </DropdownMenuContent>
              </DropdownMenu>
            ) : null}
          </CardHeader>
          <CardContent>
            <div 
              className={`text-sm font-small truncate mb-2 ${
                isOwned ? 'cursor-pointer hover:text-blue-600 dark:hover:text-blue-400' : ''
              }`}
              onClick={isOwned ? handleDidClick : undefined}
              title={isOwned ? "Click to view profile" : undefined}
            >
              {didAsset.asset_id}
            </div>
          </CardContent>
        </Card>

        {isOwned && (
          <>
            <Dialog
              open={renameOpen}
              onOpenChange={(open) => !open && setRenameOpen(false)}
            >
              <DialogContent>
                <DialogHeader>
                  <DialogTitle>
                    <Trans>Rename Profile</Trans>
                  </DialogTitle>
                  <DialogDescription>
                    <Trans>Enter the new display name for this profile.</Trans>
                  </DialogDescription>
                </DialogHeader>
                <div className='grid w-full items-center gap-4'>
                  <div className='flex flex-col space-y-1.5'>
                    <Label htmlFor='name'>
                      <Trans>Name</Trans>
                    </Label>
                    <Input
                      id='name'
                      placeholder={t`Profile name`}
                      value={name}
                      onChange={(event) => setName(event.target.value)}
                      onKeyDown={(event) => {
                        if (event.key === 'Enter') {
                          event.preventDefault();
                          rename();
                        }
                      }}
                    />
                  </div>
                </div>
                <DialogFooter className='gap-2'>
                  <Button
                    variant='outline'
                    onClick={() => {
                      setRenameOpen(false);
                      setName('');
                    }}
                  >
                    <Trans>Cancel</Trans>
                  </Button>
                  <Button onClick={rename} disabled={!name}>
                    <Trans>Rename</Trans>
                  </Button>
                </DialogFooter>
              </DialogContent>
            </Dialog>

            <TransferDialog
              title={t`Transfer Profile`}
              open={transferOpen}
              setOpen={setTransferOpen}
              onSubmit={onTransferSubmit}
            >
              <Trans>This will send the profile to the provided address.</Trans>
            </TransferDialog>

            <FeeOnlyDialog
              title={t`Burn Profile`}
              submitButtonLabel={t`Burn`}
              open={burnOpen}
              setOpen={setBurnOpen}
              onSubmit={onBurnSubmit}
            >
              <Trans>
                This will permanently delete the profile by sending it to the
                burn address.
              </Trans>
            </FeeOnlyDialog>

            <FeeOnlyDialog
              title={t`Normalize Profile`}
              submitButtonLabel={t`Normalize`}
              open={normalizeOpen}
              setOpen={setNormalizeOpen}
              onSubmit={onNormalizeSubmit}
            >
              <Trans>
                This will modify the profile&apos;s recovery info to be
                compatible with the Chia reference wallet.
              </Trans>
            </FeeOnlyDialog>

            <ConfirmationDialog
              response={response}
              showRecipientDetails={false}
              close={() => {
                setResponse(null);
                setIsTransferring(false);
                setIsBurning(false);
                setIsNormalizing(false);
              }}
              onConfirm={() => updateDids?.()}
              additionalData={
                isTransferring && response
                  ? {
                      title: t`Transfer DID`,
                      content: (
                        <DidConfirmation
                          dids={[didRecord]}
                          address={transferAddress}
                          type='transfer'
                        />
                      ),
                    }
                  : isBurning && response
                    ? {
                        title: t`Burn DID`,
                        content: (
                          <DidConfirmation dids={[didRecord]} type='burn' />
                        ),
                      }
                    : isNormalizing && response
                      ? {
                          title: t`Normalize DID`,
                          content: (
                            <DidConfirmation
                              dids={[didRecord]}
                              type='normalize'
                            />
                          ),
                        }
                      : undefined
              }
            />
          </>
        )}
      </>
    );
  }

  return (
    <div className={`flex items-center gap-2 ${className}`}>
      <AssetIcon asset={didAsset} size='md' />
      <div className='flex-1 min-w-0'>
        <div className='font-medium truncate'>{didAsset.name}</div>
      </div>
      {!mintGardenProfile.is_unknown && (
        <Button
          variant='ghost'
          size='sm'
          onClick={handleMintGardenClick}
          className='text-blue-700 dark:text-blue-300 hover:underline p-0 h-auto font-normal'
        >
          <Trans>View Profile</Trans>
          <ExternalLinkIcon className='w-3 h-3 ml-1' />
        </Button>
      )}
    </div>
  );
}

export default ProfileCard;
