import { commands, NftData, NftRecord, TransactionResponse } from '@/bindings';
import { useErrors } from '@/hooks/useErrors';
import useOfferStateWithDefault from '@/hooks/useOfferStateWithDefault';
import { toMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import {
  ChevronDown,
  Flame,
  HandCoins,
  SendIcon,
  UserRoundPlus,
} from 'lucide-react';
import { useEffect, useState } from 'react';
import { toast } from 'react-toastify';
import { AssignNftDialog } from './AssignNftDialog';
import ConfirmationDialog from './ConfirmationDialog';
import { FeeOnlyDialog } from './FeeOnlyDialog';
import { TransferDialog } from './TransferDialog';
import { Button } from './ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from './ui/dropdown-menu';
import { NftConfirmation } from './confirmations/NftConfirmation';
import { useNavigate } from 'react-router-dom';

export interface MultiSelectActionsProps {
  selected: string[];
  nfts?: NftRecord[];
  nftData?: Record<string, NftData | null>;
  onConfirm: () => void;
}

export function MultiSelectActions({
  selected,
  nfts: propNfts,
  nftData: propNftData,
  onConfirm,
}: MultiSelectActionsProps) {
  const walletState = useWalletState();
  const [offerState, setOfferState] = useOfferStateWithDefault();

  const { addError } = useErrors();
  const navigate = useNavigate();

  const [transferOpen, setTransferOpen] = useState(false);
  const [assignOpen, setAssignOpen] = useState(false);
  const [burnOpen, setBurnOpen] = useState(false);
  const [isBurning, setIsBurning] = useState(false);
  const [isTransferring, setIsTransferring] = useState(false);
  const [isEditingProfile, setIsEditingProfile] = useState(false);
  const [transferAddress, setTransferAddress] = useState('');
  const [assignedProfileId, setAssignedProfileId] = useState<string | null>(
    null,
  );
  const [response, setResponse] = useState<TransactionResponse | null>(null);

  // State for fetched NFT data when props aren't provided
  const [fetchedNfts, setFetchedNfts] = useState<NftRecord[]>([]);
  const [fetchedNftData, setFetchedNftData] = useState<
    Record<string, NftData | null>
  >({});

  // Use prop values if provided, otherwise use fetched values
  const nfts = propNfts?.length ? propNfts : fetchedNfts;
  const nftData = Object.keys(propNftData || {}).length
    ? propNftData || {}
    : fetchedNftData;

  const selectedCount = selected.length;

  // Fetch NFT records if not provided as props
  useEffect(() => {
    // Skip if we already have NFT records from props or if there are no selected NFTs
    if ((propNfts && propNfts.length > 0) || selected.length === 0) return;

    const fetchNftRecords = async () => {
      try {
        const records: NftRecord[] = [];
        for (const nftId of selected) {
          const response = await commands.getNft({ nft_id: nftId });
          if (response.nft) {
            records.push(response.nft);
          }
        }
        setFetchedNfts(records);
      } catch (error: any) {
        addError(error);
      }
    };

    fetchNftRecords();
  }, [selected, propNfts, addError]);

  // Fetch NFT data if not provided as props
  useEffect(() => {
    // Skip if we already have NFT data from props or if there are no NFT records
    if (
      (propNftData && Object.keys(propNftData).length > 0) ||
      (fetchedNfts.length === 0 && (!propNfts || propNfts.length === 0))
    )
      return;

    const nftsToFetch = propNfts?.length ? propNfts : fetchedNfts;

    const fetchNftData = async () => {
      try {
        const data: Record<string, NftData | null> = {};
        for (const nft of nftsToFetch) {
          const response = await commands.getNftData({
            nft_id: nft.launcher_id,
          });
          data[nft.launcher_id] = response.data;
        }
        setFetchedNftData(data);
      } catch (error: any) {
        addError(error);
      }
    };

    fetchNftData();
  }, [fetchedNfts, propNfts, propNftData, addError]);

  const onTransferSubmit = (address: string, fee: string) => {
    setIsTransferring(true);
    setTransferAddress(address);
    commands
      .transferNfts({
        nft_ids: selected,
        address,
        fee: toMojos(fee, walletState.sync.unit.decimals),
      })
      .then(setResponse)
      .catch((err: any) => {
        setIsTransferring(false);
        addError(err);
      })
      .finally(() => setTransferOpen(false));
  };

  const onAssignSubmit = (profile: string | null, fee: string) => {
    setIsEditingProfile(true);
    setAssignedProfileId(profile);
    commands
      .assignNftsToDid({
        nft_ids: selected,
        did_id: profile,
        fee: toMojos(fee, walletState.sync.unit.decimals),
      })
      .then(setResponse)
      .catch((err: any) => {
        setIsEditingProfile(false);
        addError(err);
      })
      .finally(() => setAssignOpen(false));
  };

  const onBurnSubmit = (fee: string) => {
    setIsBurning(true);
    commands
      .transferNfts({
        nft_ids: selected,
        address: walletState.sync.burn_address,
        fee: toMojos(fee, walletState.sync.unit.decimals),
      })
      .then(setResponse)
      .catch((err: any) => {
        setIsBurning(false);
        addError(err);
      })
      .finally(() => setBurnOpen(false));
  };

  return (
    <>
      <div
        className='absolute flex justify-between items-center gap-3 bottom-6 w-60 px-5 p-3 rounded-lg shadow-md shadow-black/20 left-1/2 -translate-x-1/2 bg-white border border-neutral-200 dark:border-neutral-800 dark:bg-neutral-900'
        role='region'
        aria-label={t`Selected NFTs actions`}
      >
        <span
          className='flex-shrink-0 text-neutral-900 dark:text-white'
          aria-live='polite'
        >
          <Trans>{selectedCount} selected</Trans>
        </span>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              className='flex items-center gap-1'
              aria-label={t`Actions for ${selectedCount} selected NFTs`}
            >
              <Trans>Actions</Trans>
              <ChevronDown className='h-5 w-5' aria-hidden='true' />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align='center'>
            <DropdownMenuGroup>
              <DropdownMenuItem
                className='cursor-pointer'
                onClick={(e) => {
                  e.stopPropagation();
                  setTransferOpen(true);
                }}
                aria-label={t`Transfer ${selectedCount} selected NFTs`}
              >
                <SendIcon className='mr-2 h-4 w-4' aria-hidden='true' />
                <span>
                  <Trans>Transfer</Trans>
                </span>
              </DropdownMenuItem>

              <DropdownMenuItem
                className='cursor-pointer'
                onClick={(e) => {
                  e.stopPropagation();
                  setAssignOpen(true);
                }}
                aria-label={t`Edit profile for ${selectedCount} selected NFTs`}
              >
                <UserRoundPlus className='mr-2 h-4 w-4' aria-hidden='true' />
                <span>
                  <Trans>Edit Profile</Trans>
                </span>
              </DropdownMenuItem>

              <DropdownMenuItem
                className='cursor-pointer'
                onClick={(e) => {
                  e.stopPropagation();
                  setBurnOpen(true);
                }}
                aria-label={t`Burn ${selectedCount} selected NFTs`}
              >
                <Flame className='mr-2 h-4 w-4' aria-hidden='true' />
                <span>
                  <Trans>Burn</Trans>
                </span>
              </DropdownMenuItem>

              <DropdownMenuSeparator />

              <DropdownMenuItem
                className='cursor-pointer'
                onClick={(e) => {
                  e.stopPropagation();

                  const newNfts = [...offerState.offered.nfts];
                  let addedCount = 0;

                  for (const item of selected) {
                    if (newNfts.includes(item)) {
                      continue;
                    }

                    newNfts.push(item);
                    addedCount++;
                  }

                  setOfferState({
                    offered: {
                      ...offerState.offered,
                      nfts: newNfts,
                    },
                  });

                  const nfts = addedCount === 1 ? t`NFT` : t`NFTs`;
                  const message =
                    addedCount > 0
                      ? t`Added ${addedCount} ${nfts} to offer`
                      : t`Selected NFTs are already in the offer`;
                  toast.success(message, {
                    onClick: () => navigate('/offers/make'),
                  });

                  onConfirm();
                }}
                aria-label={t`Add ${selectedCount} selected NFTs to offer`}
              >
                <HandCoins className='mr-2 h-4 w-4' aria-hidden='true' />
                <span>
                  <Trans>Add to Offer</Trans>
                </span>
              </DropdownMenuItem>
            </DropdownMenuGroup>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      <TransferDialog
        title={t`Bulk Transfer NFTs`}
        open={transferOpen}
        setOpen={setTransferOpen}
        onSubmit={onTransferSubmit}
        aria-describedby='bulk-transfer-description'
      >
        <p id='bulk-transfer-description'>
          <Trans>
            This will bulk transfer {selectedCount} NFTs to another wallet. Are
            you sure you want to proceed?
          </Trans>
        </p>
      </TransferDialog>
      <AssignNftDialog
        title={t`Edit Profile`}
        open={assignOpen}
        setOpen={setAssignOpen}
        onSubmit={onAssignSubmit}
        aria-describedby='bulk-assign-description'
      >
        <p id='bulk-assign-description'>
          <Trans>This will bulk assign the NFTs to the selected profile.</Trans>
        </p>
      </AssignNftDialog>
      <FeeOnlyDialog
        title={t`Bulk Burn NFTs`}
        open={burnOpen}
        setOpen={setBurnOpen}
        onSubmit={onBurnSubmit}
        submitButtonLabel={t`Burn`}
        aria-describedby='bulk-burn-description'
      >
        <p id='bulk-burn-description'>
          <Trans>
            This will bulk burn {selectedCount} NFTs. This cannot be undone. Are
            you sure you want to proceed?
          </Trans>
        </p>
      </FeeOnlyDialog>
      <ConfirmationDialog
        response={response}
        showRecipientDetails={false}
        close={() => {
          setResponse(null);
          setIsBurning(false);
          setIsTransferring(false);
          setIsEditingProfile(false);
          onConfirm();
        }}
        additionalData={
          isBurning && response && nfts.length > 0
            ? {
                title: t`Burn NFT`,
                content: (
                  <NftConfirmation type='burn' nfts={nfts} nftData={nftData} />
                ),
              }
            : isTransferring && response && nfts.length > 0
              ? {
                  title: t`Transfer NFT`,
                  content: (
                    <NftConfirmation
                      type='transfer'
                      nfts={nfts}
                      nftData={nftData}
                      address={transferAddress}
                    />
                  ),
                }
              : isEditingProfile && response && nfts.length > 0
                ? {
                    title: t`Edit NFT Profile`,
                    content: (
                      <NftConfirmation
                        type='edit'
                        nfts={nfts}
                        nftData={nftData}
                        profileId={assignedProfileId}
                      />
                    ),
                  }
                : undefined
        }
      />
    </>
  );
}
