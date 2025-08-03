import {
  DidRecord,
  NftCollectionRecord,
  NftRecord,
  commands,
} from '@/bindings';
import { MintGardenProfile } from '@/components/ProfileCard';
import { NftGroupMode } from '@/hooks/useNftParams';
import useOfferStateWithDefault from '@/hooks/useOfferStateWithDefault';
import { getMintGardenProfile } from '@/lib/marketplaces';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { openUrl } from '@tauri-apps/plugin-opener';
import {
  Copy,
  ExternalLink,
  EyeIcon,
  EyeOff,
  HandCoins,
  LibraryBig,
  MoreVertical,
  Paintbrush,
  ScrollText,
  UserIcon,
} from 'lucide-react';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { toast } from 'react-toastify';
import { Button } from './ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from './ui/dropdown-menu';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from './ui/tooltip';

interface NftGroupCardProps {
  type: 'collection' | 'did';
  groupMode?: NftGroupMode;
  item: NftCollectionRecord | DidRecord;
  updateNfts: (page: number) => void;
  page: number;
  onToggleVisibility?: () => void;
  isLoading?: boolean;
  error?: Error;
  isPlaceHolder?: boolean;
  setSplitNftOffers?: (value: boolean) => void;
}

export function NftGroupCard({
  type,
  groupMode,
  item,
  updateNfts,
  page,
  onToggleVisibility,
  isLoading,
  error,
  isPlaceHolder = false,
  setSplitNftOffers,
}: NftGroupCardProps) {
  const navigate = useNavigate();
  const [offerState, setOfferState] = useOfferStateWithDefault();
  const isCollection = type === 'collection';

  // Profile state for DID cards
  const [didProfile, setDidProfile] = useState<MintGardenProfile | null>(null);

  // Fetch profile data for DID cards
  useEffect(() => {
    if (!isCollection && isDidRecord(item)) {
      getMintGardenProfile(item.launcher_id).then(setDidProfile);
    }
  }, [isCollection, item]);
  // Type guards to help TypeScript narrow the types
  const isDidRecord = (
    item: NftCollectionRecord | DidRecord,
  ): item is DidRecord => {
    return 'launcher_id' in item;
  };

  const isCollectionRecord = (
    item: NftCollectionRecord | DidRecord,
  ): item is NftCollectionRecord => {
    return 'collection_id' in item;
  };

  const getLinkPath = () => {
    if (isCollectionRecord(item)) {
      return `/nfts/collections/${item.collection_id}`;
    }
    if (isDidRecord(item)) {
      return groupMode === NftGroupMode.OwnerDid
        ? `/nfts/owners/${item.launcher_id}`
        : `/nfts/minters/${item.launcher_id}`;
    }
    return '';
  };

  const getDefaultName = () => {
    if (isCollection) {
      return t`Unnamed Collection`;
    }

    return groupMode === NftGroupMode.OwnerDid ? (
      <Trans>Untitled Profile</Trans>
    ) : (
      <Trans>Unknown Minter</Trans>
    );
  };

  const getId = () => {
    if (isCollectionRecord(item)) {
      return item.collection_id;
    }
    if (isDidRecord(item)) {
      return item.launcher_id;
    }
    return '';
  };

  if (error) {
    return (
      <div
        className='border border-red-200 dark:border-red-800 rounded-lg p-4'
        role='alert'
        aria-live='polite'
      >
        <p className='text-red-600 dark:text-red-400'>{error.message}</p>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div
        className='border border-neutral-200 dark:border-neutral-800 rounded-lg p-4 animate-pulse'
        role='status'
        aria-busy='true'
        aria-label={isCollection ? t`Loading collection` : t`Loading profile`}
      >
        <div
          className='aspect-square bg-neutral-100 dark:bg-neutral-800 rounded-t-lg'
          aria-hidden='true'
        />
        <div
          className='h-6 bg-neutral-100 dark:bg-neutral-800 rounded mt-4'
          aria-hidden='true'
        />
      </div>
    );
  }

  // Use profile name for DID cards, fallback to item name or default
  const cardName =
    !isCollection && didProfile
      ? didProfile.name
      : item.name || getDefaultName();

  const cardId = getId();

  return (
    <div
      onClick={() => {
        updateNfts(page);
        navigate(getLinkPath());
      }}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          updateNfts(page);
          navigate(getLinkPath());
        }
      }}
      role='article'
      tabIndex={0}
      className={`cursor-pointer group border border-neutral-200 dark:border-neutral-800 rounded-lg focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary ${
        isCollection && !item.visible ? 'opacity-50' : ''
      }`}
      aria-label={
        isCollection
          ? t`Collection ${cardName}`
          : groupMode === NftGroupMode.OwnerDid
            ? t`Profile ${cardName}`
            : t`Minter ${cardName}`
      }
      aria-current={window.location.pathname === getLinkPath()}
    >
      <div className='overflow-hidden rounded-t-lg relative' aria-hidden='true'>
        {isCollection ? (
          <div
            className='bg-neutral-100 dark:bg-neutral-800 flex items-center justify-center aspect-square'
            aria-hidden='true'
          >
            {isCollectionRecord(item) && item.icon ? (
              <img
                src={item.icon}
                alt={(() => {
                  const name = item.name || '';
                  return t`Icon for ${name}`;
                })()}
                className='object-cover h-full w-full'
                aria-hidden='true'
                loading='lazy'
              />
            ) : (
              <LibraryBig
                className='h-12 w-12 text-neutral-400 dark:text-neutral-600'
                aria-hidden='true'
              />
            )}
          </div>
        ) : (
          <div
            className='bg-neutral-100 dark:bg-neutral-800 flex items-center justify-center aspect-square'
            aria-hidden='true'
          >
            {didProfile?.avatar_uri ? (
              <img
                src={didProfile.avatar_uri}
                alt={`${cardName} avatar`}
                className='object-cover h-full w-full'
                aria-hidden='true'
                loading='lazy'
              />
            ) : groupMode === NftGroupMode.OwnerDid ? (
              <UserIcon
                className='h-12 w-12 text-neutral-400 dark:text-neutral-600'
                aria-hidden='true'
              />
            ) : (
              <Paintbrush
                className='h-12 w-12 text-neutral-400 dark:text-neutral-600'
                aria-hidden='true'
              />
            )}
          </div>
        )}
      </div>
      <div className='border-t border-neutral-200 bg-white text-neutral-950 shadow dark:border-neutral-800 dark:bg-neutral-900 dark:text-neutral-50 text-md flex items-center justify-between rounded-b-lg p-2 pl-3'>
        <span className='truncate'>
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <h3 className='font-medium leading-none truncate block'>
                  {cardName}
                </h3>
              </TooltipTrigger>
              <TooltipContent>
                <p>{cardName}</p>
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>

          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <p
                  className='text-xs text-muted-foreground truncate'
                  aria-label={t`ID: ${cardId}`}
                >
                  {cardId}
                </p>
              </TooltipTrigger>
              <TooltipContent>
                <p>{cardId}</p>
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
        </span>

        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              variant='ghost'
              size='icon'
              aria-label={t`Options for ${cardName}`}
              onClick={(e) => e.stopPropagation()}
            >
              <MoreVertical className='h-5 w-5' aria-hidden='true' />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align='end'>
            <DropdownMenuGroup>
              {isCollection && (
                <>
                  <DropdownMenuItem
                    className='cursor-pointer'
                    disabled={isPlaceHolder}
                    onClick={(e) => {
                      e.stopPropagation();
                      navigate(`/nfts/collections/${cardId}/metadata`);
                    }}
                    aria-label={t`View ${cardName} Metadata`}
                  >
                    <ScrollText className='mr-2 h-4 w-4' aria-hidden='true' />
                    <span>
                      <Trans>View Metadata</Trans>
                    </span>
                  </DropdownMenuItem>
                  <DropdownMenuItem
                    className='cursor-pointer'
                    disabled={isPlaceHolder}
                    onClick={(e) => {
                      e.stopPropagation();
                      openUrl(`https://mintgarden.io/collections/${cardId}`);
                    }}
                    aria-label={t`View ${cardName} on Mintgarden`}
                  >
                    <ExternalLink className='mr-2 h-4 w-4' aria-hidden='true' />
                    <span>
                      <Trans>View on Mintgarden</Trans>
                    </span>
                  </DropdownMenuItem>

                  <DropdownMenuItem
                    className='cursor-pointer'
                    onClick={(e) => {
                      e.stopPropagation();
                      onToggleVisibility?.();
                    }}
                    disabled={isPlaceHolder}
                    aria-label={
                      item.visible ? t`Hide ${cardName}` : t`Show ${cardName}`
                    }
                  >
                    {item.visible ? (
                      <>
                        <EyeOff className='mr-2 h-4 w-4' aria-hidden='true' />
                        <span>
                          <Trans>Hide</Trans>
                        </span>
                      </>
                    ) : (
                      <>
                        <EyeIcon className='mr-2 h-4 w-4' aria-hidden='true' />
                        <span>
                          <Trans>Show</Trans>
                        </span>
                      </>
                    )}
                  </DropdownMenuItem>
                </>
              )}

              {!isCollection && (
                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    openUrl(`https://mintgarden.io/${cardId}`);
                  }}
                  aria-label={t`View ${cardName} on Mintgarden`}
                >
                  <ExternalLink className='mr-2 h-4 w-4' aria-hidden='true' />
                  <span>
                    <Trans>View on Mintgarden</Trans>
                  </span>
                </DropdownMenuItem>
              )}

              <DropdownMenuItem
                className='cursor-pointer'
                onClick={(e) => {
                  e.stopPropagation();
                  writeText(cardId);
                  toast.success(
                    isCollection
                      ? t`Collection ID copied to clipboard`
                      : groupMode === NftGroupMode.OwnerDid
                        ? t`Profile ID copied to clipboard`
                        : t`Minter ID copied to clipboard`,
                  );
                }}
                aria-label={t`Copy ${cardName} ID to clipboard`}
              >
                <Copy className='mr-2 h-4 w-4' aria-hidden='true' />
                <span>
                  <Trans>Copy ID</Trans>
                </span>
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem
                className='cursor-pointer'
                onClick={async (e) => {
                  e.stopPropagation();
                  try {
                    let nftIds: string[] = [];

                    if (isCollectionRecord(item)) {
                      // Fetch all NFTs in the collection
                      const response = await commands.getNfts({
                        collection_id: item.collection_id,
                        minter_did_id: null,
                        owner_did_id: null,
                        name: null,
                        offset: 0,
                        limit: 1000, // Use a large limit to get all NFTs
                        sort_mode: 'name',
                        include_hidden: true,
                      });
                      nftIds = response.nfts.map(
                        (nft: NftRecord) => nft.launcher_id,
                      );
                    } else if (isDidRecord(item)) {
                      // Fetch all NFTs for the DID
                      const response = await commands.getNfts({
                        collection_id: null,
                        minter_did_id:
                          groupMode === NftGroupMode.MinterDid
                            ? item.launcher_id
                            : null,
                        owner_did_id:
                          groupMode === NftGroupMode.OwnerDid
                            ? item.launcher_id
                            : null,
                        name: null,
                        offset: 0,
                        limit: 1000, // Use a large limit to get all NFTs
                        sort_mode: 'name',
                        include_hidden: true,
                      });
                      nftIds = response.nfts.map(
                        (nft: NftRecord) => nft.launcher_id,
                      );
                    }

                    const newNfts = [...offerState.offered.nfts];
                    let addedCount = 0;

                    for (const nftId of nftIds) {
                      if (newNfts.includes(nftId)) {
                        continue;
                      }

                      newNfts.push(nftId);
                      addedCount++;
                    }

                    setOfferState({
                      offered: {
                        ...offerState.offered,
                        nfts: newNfts,
                      },
                    });

                    if (setSplitNftOffers) {
                      setSplitNftOffers(true);
                    }

                    const nfts = addedCount === 1 ? t`NFT` : t`NFTs`;
                    const message =
                      addedCount > 0
                        ? t`Added ${addedCount} ${nfts} to offer`
                        : t`Selected NFTs are already in the offer`;
                    toast.success(message, {
                      onClick: () =>
                        navigate('/offers/make', {
                          state: { splitNftOffers: true },
                        }),
                    });
                  } catch (error: unknown) {
                    toast.error(
                      error instanceof Error
                        ? error.message
                        : t`Failed to add NFTs to offer`,
                    );
                  }
                }}
                aria-label={t`Add all NFTs in ${cardName} to an offer`}
              >
                <HandCoins className='mr-2 h-4 w-4' aria-hidden='true' />
                <span title={t`Add all NFTs in ${cardName} to an offer`}>
                  <Trans>Add All to Offer</Trans>
                </span>
              </DropdownMenuItem>
            </DropdownMenuGroup>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </div>
  );
}
