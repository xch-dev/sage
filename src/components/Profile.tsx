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
import { toMojos } from '@/lib/utils';
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
import { useEffect, useMemo, useState } from 'react';
import { toast } from 'react-toastify';
import { commands, DidRecord, TransactionResponse } from '../bindings';
import { CustomError } from '../contexts/ErrorContext';

export interface MintGardenProfile {
  encoded_id: string;
  name: string;
  avatar_uri: string | null;
  is_unknown: boolean;
}

export interface ProfileProps {
  // For MintGarden profiles (DID string only)
  did?: string;
  // For local DID records (with enhanced MintGarden data)
  didRecord?: DidRecord;
  // Common props
  variant?: 'default' | 'compact' | 'card';
  showDid?: boolean;
  className?: string;
  onProfileLoad?: (profile: MintGardenProfile) => void;
  // Local DID specific
  updateDids?: () => void;
  showMintGardenProfile?: boolean;
}

// ProfileContent component to handle the actual rendering
interface ProfileContentProps {
  didRecord?: DidRecord;
  mintGardenProfile: MintGardenProfile;
  isMintGardenLoading: boolean;
  variant: 'default' | 'compact' | 'card';
  showDid: boolean;
  className: string;
  showMintGardenProfile: boolean;
  name: string;
  setName: (name: string) => void;
  response: TransactionResponse | null;
  renameOpen: boolean;
  setRenameOpen: (open: boolean) => void;
  transferOpen: boolean;
  setTransferOpen: (open: boolean) => void;
  burnOpen: boolean;
  setBurnOpen: (open: boolean) => void;
  normalizeOpen: boolean;
  setNormalizeOpen: (open: boolean) => void;
  isTransferring: boolean;
  isBurning: boolean;
  isNormalizing: boolean;
  transferAddress: string;
  updateDids?: () => void;
  addError: (error: CustomError) => void;
  walletState: { sync: { unit: { decimals: number }; burn_address: string } };
  setResponse: (response: TransactionResponse | null) => void;
  setIsTransferring: (transferring: boolean) => void;
  setIsBurning: (burning: boolean) => void;
  setIsNormalizing: (normalizing: boolean) => void;
  setTransferAddress: (address: string) => void;
}

function ProfileContent({
  didRecord,
  mintGardenProfile,
  isMintGardenLoading,
  variant,
  showDid,
  className,
  name,
  setName,
  response,
  renameOpen,
  setRenameOpen,
  transferOpen,
  setTransferOpen,
  burnOpen,
  setBurnOpen,
  normalizeOpen,
  setNormalizeOpen,
  isTransferring,
  isBurning,
  isNormalizing,
  transferAddress,
  updateDids,
  addError,
  walletState,
  setResponse,
  setIsTransferring,
  setIsBurning,
  setIsNormalizing,
  setTransferAddress,
}: ProfileContentProps) {
  // Local DID action handlers
  const rename = () => {
    if (!name || !didRecord) return;

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
    if (!didRecord) return;

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
    if (!didRecord) return;

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
    if (!didRecord) return;

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
    if (!didRecord) return;

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

  // Display logic
  const displayName = !mintGardenProfile.is_unknown
    ? mintGardenProfile.name
    : didRecord?.name || mintGardenProfile?.name || t`Untitled Profile`;

  const displayDid = mintGardenProfile.encoded_id || '';

  // Loading state
  if (isMintGardenLoading) {
    return (
      <div className={`flex items-center gap-2 ${className}`}>
        <Skeleton className='w-8 h-8 rounded-full' />
        <div className='space-y-1'>
          <Skeleton className='h-4 w-24' />
          {showDid && <Skeleton className='h-3 w-32' />}
        </div>
      </div>
    );
  }

  if (variant === 'compact') {
    return (
      <div className={`flex items-center gap-2 ${className}`}>
        <AssetIcon
          iconUrl={mintGardenProfile.avatar_uri}
          size='sm'
          kind='did'
        />
        <span className='text-sm font-medium truncate'>{displayName}</span>
        {showDid && (
          <span className='text-xs text-gray-500 font-mono truncate'>
            {displayDid}
          </span>
        )}
        {!mintGardenProfile.is_unknown && (
          <Button
            variant='ghost'
            size='sm'
            onClick={handleMintGardenClick}
            className='p-1 h-auto'
            title={`View ${displayName} on MintGarden`}
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
            didRecord && !didRecord.visible
              ? 'opacity-50 grayscale'
              : didRecord && didRecord.created_height === null
                ? 'pulsate-opacity'
                : ''
          }`}
        >
          <CardHeader className='-mt-2 flex flex-row items-center justify-between space-y-0 pb-2 pr-2 space-x-2'>
            <CardTitle className='text-md font-medium truncate flex items-center'>
              <AssetIcon
                iconUrl={mintGardenProfile.avatar_uri}
                size='md'
                kind='did'
                className='mr-2'
              />
              {displayName}
            </CardTitle>
            {didRecord ? (
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
            <div className='text-sm font-small truncate mb-2'>{displayDid}</div>
          </CardContent>
        </Card>

        {didRecord && (
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
      <AssetIcon iconUrl={mintGardenProfile.avatar_uri} size='md' kind='did' />
      <div className='flex-1 min-w-0'>
        <div className='font-medium truncate'>{displayName}</div>
        {showDid && (
          <div className='text-xs text-gray-500 font-mono truncate'>
            {displayDid}
          </div>
        )}
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

// Enhanced Unified Profile Component
export function Profile({
  did,
  didRecord,
  variant = 'default',
  showDid = false,
  className = '',
  showMintGardenProfile = true,
  onProfileLoad,
  updateDids,
}: ProfileProps) {
  const { addError } = useErrors();
  const walletState = useWalletState();

  // Determine the DID to use for MintGarden lookup
  const didToLookup = didRecord?.launcher_id || did;

  // State for MintGarden profile data
  const [mintGardenProfile, setMintGardenProfile] = useState<MintGardenProfile>(
    {
      encoded_id: didToLookup || '',
      name:
        didRecord?.name ||
        `${(didToLookup || '').slice(9, 19)}...${(didToLookup || '').slice(-4)}`,
      avatar_uri: null,
      is_unknown: true,
    },
  );
  const [isMintGardenLoading, setIsMintGardenLoading] = useState(false);

  // State for local DID actions (only when didRecord is provided)
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
    if (!didToLookup) return;

    if (showMintGardenProfile) {
      setIsMintGardenLoading(true);
      getMintGardenProfile(didToLookup)
        .then((profileData) => {
          setMintGardenProfile(profileData);
          onProfileLoad?.(profileData);
        })
        .catch(() => {
          // Create fallback profile for failed lookups
          setMintGardenProfile({
            encoded_id: didToLookup,
            name:
              didRecord?.name ||
              `${didToLookup.slice(9, 19)}...${didToLookup.slice(-4)}`,
            avatar_uri: null,
            is_unknown: true,
          });
        })
        .finally(() => {
          setIsMintGardenLoading(false);
        });
    } else {
      setMintGardenProfile({
        encoded_id: didToLookup,
        name:
          didRecord?.name ||
          `${didToLookup.slice(9, 19)}...${didToLookup.slice(-4)}`,
        avatar_uri: null,
        is_unknown: true,
      });
    }
  }, [didToLookup, didRecord?.name, onProfileLoad, showMintGardenProfile]);

  return (
    <ProfileContent
      didRecord={didRecord}
      mintGardenProfile={mintGardenProfile}
      isMintGardenLoading={isMintGardenLoading}
      variant={variant}
      showDid={showDid}
      className={className}
      showMintGardenProfile={showMintGardenProfile}
      name={name}
      setName={setName}
      response={response}
      renameOpen={renameOpen}
      setRenameOpen={setRenameOpen}
      transferOpen={transferOpen}
      setTransferOpen={setTransferOpen}
      burnOpen={burnOpen}
      setBurnOpen={setBurnOpen}
      normalizeOpen={normalizeOpen}
      setNormalizeOpen={setNormalizeOpen}
      isTransferring={isTransferring}
      isBurning={isBurning}
      isNormalizing={isNormalizing}
      transferAddress={transferAddress}
      updateDids={updateDids}
      addError={addError}
      walletState={walletState}
      setResponse={setResponse}
      setIsTransferring={setIsTransferring}
      setIsBurning={setIsBurning}
      setIsNormalizing={setIsNormalizing}
      setTransferAddress={setTransferAddress}
    />
  );
}

// Hook for managing MintGarden profile state
export function useMintGardenProfile(did: string | undefined) {
  const defaultProfile = useMemo(
    () => ({
      encoded_id: did ?? '',
      name: `${did?.slice(9, 19)}...${did?.slice(-4)}`,
      avatar_uri: null,
      is_unknown: true,
    }),
    [did],
  );

  const [profile, setProfile] = useState<MintGardenProfile>(defaultProfile);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!did) {
      setProfile(defaultProfile);
      setIsLoading(false);
      return;
    }

    setIsLoading(true);
    setError(null);

    getMintGardenProfile(did)
      .then((profileData) => {
        setProfile(profileData);
      })
      .finally(() => {
        setIsLoading(false);
      });
  }, [did, defaultProfile]);

  return { profile, isLoading, error };
}

// Clickable profile component that opens MintGarden on click
export interface ClickableProfileProps extends ProfileProps {
  onClick?: (profile: MintGardenProfile) => void;
}

export function ClickableProfile({ onClick, ...props }: ClickableProfileProps) {
  const handleProfileLoad = (profile: MintGardenProfile) => {
    props.onProfileLoad?.(profile);
  };

  const handleClick = () => {
    const didToUse = props.didRecord?.launcher_id || props.did;
    if (onClick && didToUse) {
      getMintGardenProfile(didToUse).then(onClick);
    } else if (didToUse) {
      openUrl(`https://mintgarden.io/${didToUse}`);
    }
  };

  return (
    <div
      className='cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-800 rounded p-1 transition-colors'
      onClick={handleClick}
    >
      <Profile {...props} onProfileLoad={handleProfileLoad} />
    </div>
  );
}

export default Profile;
