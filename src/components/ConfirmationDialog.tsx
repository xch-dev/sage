import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { useWalletState } from '@/state';
import BigNumber from 'bignumber.js';
import {
  BadgeMinus,
  BadgePlus,
  ForwardIcon,
  LoaderCircleIcon,
} from 'lucide-react';
import { PropsWithChildren, useState } from 'react';
import { commands, Error, TransactionSummary } from '../bindings';
import { Badge } from './ui/badge';

export interface ConfirmationDialogProps {
  summary: TransactionSummary | null;
  close: () => void;
  onConfirm?: () => void;
  onError?: (error: Error) => void;
}

interface SpentCoin {
  sort: number;
  badge: string;
  label: string;
  coinId: string;
}

interface CreatedCoin {
  sort: number;
  badge: string;
  label: string;
  address: string;
}

export default function ConfirmationDialog({
  summary,
  close,
  onConfirm,
  onError,
}: ConfirmationDialogProps) {
  const walletState = useWalletState();

  const [pending, setPending] = useState(false);

  const spent: Array<SpentCoin> = [];
  const created: Array<CreatedCoin> = [];

  if (summary) {
    for (const input of summary.inputs || []) {
      if (input.type === 'xch') {
        const ticker = walletState.sync.unit.ticker;

        spent.push({
          badge: 'Chia',
          label: `${input.amount} ${ticker}`,
          coinId: input.coin_id,
          sort: 1,
        });

        for (const output of input.outputs) {
          if (summary.inputs.find((i) => i.coin_id === output.coin_id)) {
            continue;
          }

          created.push({
            badge: 'Chia',
            label: `${output.amount} ${ticker}`,
            address: output.receiving ? 'Change' : output.address,
            sort: 1,
          });
        }
      }

      if (input.type === 'cat') {
        const ticker = input.ticker || 'CAT';

        spent.push({
          badge: `CAT ${input.name || input.asset_id}`,
          label: `${input.amount} ${ticker}`,
          coinId: input.coin_id,
          sort: 2,
        });

        for (const output of input.outputs) {
          if (summary.inputs.find((i) => i.coin_id === output.coin_id)) {
            continue;
          }

          created.push({
            badge: `CAT ${input.name || input.asset_id}`,
            label: `${output.amount} ${ticker}`,
            address: output.receiving ? 'Change' : output.address,
            sort: 2,
          });
        }
      }

      if (input.type === 'did') {
        if (
          !summary.inputs
            .map((i) => i.outputs)
            .flat()
            .find((o) => o.coin_id === input.coin_id)
        ) {
          spent.push({
            badge: 'Profile',
            label: input.name || 'Unnamed',
            coinId: input.coin_id,
            sort: 3,
          });
        }

        for (const output of input.outputs) {
          if (summary.inputs.find((i) => i.coin_id === output.coin_id)) {
            continue;
          }

          if (
            BigNumber(output.amount)
              .multipliedBy(BigNumber(10).pow(walletState.sync.unit.decimals))
              .mod(2)
              .isEqualTo(1)
          ) {
            created.push({
              badge: 'Profile',
              label: input.name || 'Unnamed',
              address: output.receiving ? 'You' : output.address,
              sort: 3,
            });
          }
        }
      }

      if (input.type === 'nft') {
        if (
          !summary.inputs
            .map((i) => i.outputs)
            .flat()
            .find((o) => o.coin_id === input.coin_id)
        ) {
          spent.push({
            badge: 'NFT',
            label: input.name || 'Unknown',
            coinId: input.coin_id,
            sort: 4,
          });
        }

        for (const output of input.outputs) {
          if (summary.inputs.find((i) => i.coin_id === output.coin_id)) {
            continue;
          }

          if (
            BigNumber(output.amount)
              .multipliedBy(BigNumber(10).pow(walletState.sync.unit.decimals))
              .mod(2)
              .isEqualTo(1)
          ) {
            created.push({
              badge: 'NFT',
              label: input.name || 'Unknown',
              address: output.receiving ? 'You' : output.address,
              sort: 4,
            });
          }
        }
      }
    }
  }

  return (
    <Dialog
      open={!!summary}
      onOpenChange={() => {
        close();
        setPending(false);
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Confirm transaction?</DialogTitle>
          <DialogDescription>
            <div className='mt-2 max-h-[400px] overflow-y-scroll'>
              <div className='mt-2 flex flex-col gap-1.5'>
                <Group label='Spent Coins' icon={BadgeMinus}>
                  <div className='flex flex-col gap-2'>
                    {spent
                      .sort((a, b) => a.sort - b.sort)
                      .map((spent, i) => (
                        <div className='flex flex-col gap-1 border-2 p-1.5 rounded-md'>
                          <div key={i} className='flex items-center gap-2'>
                            <Badge className='max-w-[100px]'>
                              <span className='truncate'>{spent.badge}</span>
                            </Badge>
                            <span>{spent.label}</span>
                          </div>
                          <div className='flex items-center gap-1'>
                            <ForwardIcon className='w-4 h-4' />
                            <div className='w-[250px] truncate text-neutral-600 dark:text-neutral-400'>
                              {spent.coinId}
                            </div>
                          </div>
                        </div>
                      ))}
                  </div>
                </Group>
                <Group label='Transaction Output' icon={BadgePlus}>
                  <div className='flex flex-col gap-2'>
                    {!BigNumber(summary?.fee || 0).isZero() && (
                      <div className='flex flex-col gap-1 border-2 p-1.5 rounded-md'>
                        <div className='flex items-center gap-2'>
                          <Badge>Fee</Badge>
                          <span>
                            {summary?.fee} {walletState.sync.unit.ticker}
                          </span>
                        </div>
                      </div>
                    )}
                    {created
                      .sort((a, b) => a.sort - b.sort)
                      .map((created, i) => (
                        <div className='flex flex-col gap-1 border-2 p-1.5 rounded-md'>
                          <div key={i} className='flex items-center gap-2'>
                            <Badge className='max-w-[100px]'>
                              <span className='truncate'>{created.badge}</span>
                            </Badge>
                            <span>{created.label}</span>
                          </div>
                          <div className='flex items-center gap-1'>
                            <ForwardIcon className='w-4 h-4' />
                            <div className='w-[250px] truncate text-neutral-600 dark:text-neutral-400'>
                              {created.address}
                            </div>
                          </div>
                        </div>
                      ))}
                  </div>
                </Group>
              </div>
            </div>
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button
            variant='ghost'
            onClick={() => {
              close();
              setPending(false);
            }}
          >
            Cancel
          </Button>
          <Button
            onClick={() => {
              setPending(true);

              commands.submitTransaction(summary!.data).then((result) => {
                if (result.status === 'error') {
                  close();
                  setPending(false);
                  onError?.(result.error);
                } else {
                  close();
                  setPending(false);
                  onConfirm?.();
                }
              });
            }}
            disabled={pending}
          >
            {pending && (
              <LoaderCircleIcon className='mr-2 h-4 w-4 animate-spin' />
            )}
            {pending ? 'Submitting' : 'Submit'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

interface GroupProps {
  label: string;
  icon: typeof BadgeMinus;
}

function Group({ label, icon: Icon, children }: PropsWithChildren<GroupProps>) {
  return (
    <div className='flex flex-col gap-2 w-full font-medium text-left text-neutral-900 dark:text-neutral-200 bg-neutral-100 dark:bg-neutral-900 p-2 rounded-md'>
      <div className='flex items-center gap-2 text-lg'>
        <Icon className='w-6 h-6' />
        <span>{label}</span>
      </div>
      <div>{children}</div>
    </div>
  );
}
