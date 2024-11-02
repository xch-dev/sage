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
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import BigNumber from 'bignumber.js';
import {
  BadgeMinus,
  BadgePlus,
  BoxIcon,
  CircleAlert,
  CopyCheckIcon,
  CopyIcon,
  ForwardIcon,
  LoaderCircleIcon,
} from 'lucide-react';
import { PropsWithChildren, useState } from 'react';
import { commands, Error, TransactionSummary } from '../bindings';
import { Alert, AlertDescription, AlertTitle } from './ui/alert';
import { Badge } from './ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from './ui/tabs';

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
  const [signature, setSignature] = useState<string | null>(null);

  const reset = () => {
    setPending(false);
    setSignature(null);
    close();
  };

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
            address: output.burning
              ? 'Permanently Burned'
              : output.receiving
                ? 'Change'
                : output.address,
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
            address: output.burning
              ? 'Permanently Burned'
              : output.receiving
                ? 'Change'
                : output.address,
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
              address: output.burning
                ? 'Permanently Burned'
                : output.receiving
                  ? 'You'
                  : output.address,
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
              address: output.burning
                ? 'Permanently Burned'
                : output.receiving
                  ? 'You'
                  : output.address,
              sort: 4,
            });
          }
        }
      }
    }
  }

  const json = JSON.stringify(
    {
      coin_spends: summary?.coin_spends,
      aggregated_signature:
        signature ||
        '0xc00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000',
    },
    null,
    2,
  );

  const [jsonCopied, setJsonCopied] = useState(false);

  const copyJson = () => {
    writeText(json);

    setJsonCopied(true);
    setTimeout(() => setJsonCopied(false), 2000);
  };

  return (
    <Dialog open={!!summary} onOpenChange={reset}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Confirm transaction?</DialogTitle>
          <DialogDescription>
            <div className='max-h-[360px] overflow-y-scroll'>
              <Tabs defaultValue='simple' className='mt-2'>
                <TabsList className='w-full'>
                  <TabsTrigger value='simple' className='flex-grow'>
                    Summary
                  </TabsTrigger>
                  <TabsTrigger value='advanced' className='flex-grow'>
                    Advanced
                  </TabsTrigger>
                  <TabsTrigger value='json' className='flex-grow'>
                    JSON
                  </TabsTrigger>
                </TabsList>
                <TabsContent value='simple'>
                  <Group label='Summary' icon={BadgePlus}>
                    <div className='flex flex-col gap-2'>
                      {!BigNumber(summary?.fee || 0).isZero() && (
                        <Item
                          badge='Fee'
                          label={`${summary?.fee} ${walletState.sync.unit.ticker}`}
                        />
                      )}
                      {created
                        .filter((created) => created.address !== 'Change')
                        .sort((a, b) => a.sort - b.sort)
                        .map((created, i) => (
                          <Item
                            key={i}
                            badge={created.badge}
                            label={created.label}
                            icon={ForwardIcon}
                            address={created.address}
                          />
                        ))}
                    </div>
                  </Group>
                </TabsContent>
                <TabsContent value='advanced'>
                  <div className='flex flex-col gap-1.5'>
                    <Group label='Spent Coins' icon={BadgeMinus}>
                      <div className='flex flex-col gap-2'>
                        {spent
                          .sort((a, b) => a.sort - b.sort)
                          .map((spent, i) => (
                            <Item
                              key={i}
                              badge={spent.badge}
                              label={spent.label}
                              icon={BoxIcon}
                              address={spent.coinId}
                            />
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
                            <Item
                              key={i}
                              badge={created.badge}
                              label={created.label}
                              icon={ForwardIcon}
                              address={created.address}
                            />
                          ))}
                      </div>
                    </Group>
                  </div>
                </TabsContent>
                <TabsContent value='json'>
                  <Alert>
                    <CircleAlert className='h-4 w-4' />
                    <AlertTitle>Warning</AlertTitle>
                    <AlertDescription>
                      This is the raw JSON spend bundle for this transaction. If
                      you sign it, the transaction can be submitted to the
                      mempool externally.
                    </AlertDescription>
                  </Alert>

                  <Button
                    className='mt-2'
                    onClick={() => {
                      commands
                        .signTransaction(summary!.coin_spends)
                        .then((result) => {
                          if (result.status === 'error') {
                            onError?.(result.error);
                          } else {
                            setSignature(result.data.aggregated_signature);
                          }
                        });
                    }}
                    disabled={!!signature}
                  >
                    {signature ? 'Transaction Signed' : 'Sign Transaction'}
                  </Button>

                  <div className='relative mt-2 p-2 break-all rounded-md bg-neutral-100 dark:bg-neutral-900 whitespace-pre-wrap'>
                    {json}

                    <Button
                      size='icon'
                      variant='ghost'
                      className='absolute top-2 right-2 -ml-px inline-flex items-center gap-x-1.5 rounded-md px-3 py-2 text-sm font-semibold ring-1 ring-inset ring-neutral-200 dark:ring-neutral-800 hover:bg-gray-50'
                      onClick={copyJson}
                    >
                      {jsonCopied ? (
                        <CopyCheckIcon className='-ml-0.5 h-5 w-5 text-emerald-500' />
                      ) : (
                        <CopyIcon className='-ml-0.5 h-5 w-5 text-muted-foreground' />
                      )}
                    </Button>
                  </div>
                </TabsContent>
              </Tabs>
            </div>
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant='ghost' onClick={reset}>
            Cancel
          </Button>
          <Button
            onClick={() => {
              setPending(true);

              (async () => {
                let finalSignature: string | null = signature;

                if (!finalSignature) {
                  const result = await commands.signTransaction(
                    summary!.coin_spends,
                  );
                  if (result.status === 'error') {
                    reset();
                    onError?.(result.error);
                    return;
                  }
                  finalSignature = result.data.aggregated_signature;
                }

                const result = await commands.submitTransaction({
                  coin_spends: summary!.coin_spends,
                  aggregated_signature: finalSignature,
                });

                reset();

                if (result.status === 'error') {
                  onError?.(result.error);
                } else {
                  onConfirm?.();
                }
              })().finally(() => setPending(false));
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

interface ItemProps {
  badge: string;
  label: string;
  icon?: typeof BadgeMinus;
  address?: string;
}

function Item({ badge, label, icon: Icon, address }: ItemProps) {
  return (
    <div className='flex flex-col gap-1 border-2 p-1.5 rounded-md'>
      <div className='flex items-center gap-2'>
        <Badge className='max-w-[100px]'>
          <span className='truncate'>{badge}</span>
        </Badge>
        <span>{label}</span>
      </div>
      {Icon && (
        <div className='flex items-center gap-1'>
          <Icon className='w-4 h-4' />
          <div className='w-[250px] truncate text-neutral-600 dark:text-neutral-400'>
            {address}
          </div>
        </div>
      )}
    </div>
  );
}
