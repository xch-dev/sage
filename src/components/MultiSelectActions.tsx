import { commands, TransactionResponse } from '@/bindings';
import { useErrors } from '@/hooks/useErrors';
import { useWalletState } from '@/state';
import {
  ChevronDown,
  Flame,
  SendIcon,
  UserRoundMinus,
  UserRoundPlus,
} from 'lucide-react';
import { useState } from 'react';
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
  DropdownMenuTrigger,
} from './ui/dropdown-menu';

export interface MultiSelectActionsProps {
  selected: string[];
  onConfirm: () => void;
}

export function MultiSelectActions({
  selected,
  onConfirm,
}: MultiSelectActionsProps) {
  const walletState = useWalletState();

  const { addError } = useErrors();

  const [transferOpen, setTransferOpen] = useState(false);
  const [assignOpen, setAssignOpen] = useState(false);
  const [unassignOpen, setUnassignOpen] = useState(false);
  const [burnOpen, setBurnOpen] = useState(false);
  const [response, setResponse] = useState<TransactionResponse | null>(null);

  const onTransferSubmit = (address: string, fee: string) => {
    commands
      .transferNfts({ nft_ids: selected, address, fee })
      .then(setResponse)
      .catch(addError)
      .finally(() => setTransferOpen(false));
  };

  const onAssignSubmit = (profile: string, fee: string) => {
    commands
      .assignNftsToDid({ nft_ids: selected, did_id: profile, fee })
      .then(setResponse)
      .catch(addError)
      .finally(() => setAssignOpen(false));
  };

  const onUnassignSubmit = (fee: string) => {
    commands
      .assignNftsToDid({ nft_ids: selected, did_id: null, fee })
      .then(setResponse)
      .catch(addError)
      .finally(() => setUnassignOpen(false));
  };

  const onBurnSubmit = (fee: string) => {
    commands
      .transferNfts({
        nft_ids: selected,
        address: walletState.sync.burn_address,
        fee,
      })
      .then(setResponse)
      .catch(addError)
      .finally(() => setBurnOpen(false));
  };

  return (
    <>
      <div className='absolute flex justify-between items-center gap-3 bottom-6 w-60 px-5 p-3 rounded-lg shadow-md shadow-black/20 left-1/2 -translate-x-1/2 bg-white border border-neutral-200 dark:border-neutral-800 dark:bg-neutral-900'>
        <span className='flex-shrink-0 text-neutral-900 dark:text-white'>
          {selected.length} selected
        </span>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button className='flex items-center gap-1'>
              Actions
              <ChevronDown className='h-5 w-5' />
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
              >
                <SendIcon className='mr-2 h-4 w-4' />
                <span>Transfer</span>
              </DropdownMenuItem>

              <DropdownMenuItem
                className='cursor-pointer'
                onClick={(e) => {
                  e.stopPropagation();
                  setAssignOpen(true);
                }}
              >
                <UserRoundPlus className='mr-2 h-4 w-4' />
                <span>Assign Profile</span>
              </DropdownMenuItem>

              <DropdownMenuItem
                className='cursor-pointer'
                onClick={(e) => {
                  e.stopPropagation();
                  setUnassignOpen(true);
                }}
              >
                <UserRoundMinus className='mr-2 h-4 w-4' />
                <span>Unassign Profile</span>
              </DropdownMenuItem>

              <DropdownMenuItem
                className='cursor-pointer'
                onClick={(e) => {
                  e.stopPropagation();
                  setBurnOpen(true);
                }}
              >
                <Flame className='mr-2 h-4 w-4' />
                <span>Burn</span>
              </DropdownMenuItem>
            </DropdownMenuGroup>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      <TransferDialog
        title='Bulk Transfer NFTs'
        open={transferOpen}
        setOpen={setTransferOpen}
        onSubmit={onTransferSubmit}
      >
        This will bulk transfer {selected.length} NFTs to another wallet. Are
        you sure you want to proceed?
      </TransferDialog>

      <AssignNftDialog
        title='Bulk Assign Profile'
        open={assignOpen}
        setOpen={setAssignOpen}
        onSubmit={onAssignSubmit}
      >
        This will bulk assign the NFTs to the selected profile.
      </AssignNftDialog>

      <FeeOnlyDialog
        title='Bulk Unassign Profile'
        open={unassignOpen}
        setOpen={setUnassignOpen}
        onSubmit={onUnassignSubmit}
      >
        This will bulk unassign the NFTs from their profiles.
      </FeeOnlyDialog>

      <FeeOnlyDialog
        title='Bulk Burn NFTs'
        open={burnOpen}
        setOpen={setBurnOpen}
        onSubmit={onBurnSubmit}
      >
        This will bulk burn {selected.length} NFTs. This cannot be undone. Are
        you sure you want to proceed?
      </FeeOnlyDialog>

      <ConfirmationDialog
        response={response}
        close={() => {
          setResponse(null);
          onConfirm();
        }}
      />
    </>
  );
}
