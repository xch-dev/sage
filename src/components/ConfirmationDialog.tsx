import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { useErrors } from '@/hooks/useErrors';
import { toDecimal } from '@/lib/utils';
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
import { PropsWithChildren, useEffect, useState } from 'react';
import {
  commands,
  TakeOfferResponse,
  TransactionResponse,
  TransactionSummary,
  Unit,
} from '../bindings';
import { Alert, AlertDescription, AlertTitle } from './ui/alert';
import { Badge } from './ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from './ui/tabs';
import { Trans } from '@lingui/react/macro';
import { t } from '@lingui/core/macro';

export interface ConfirmationDialogProps {
  response: TransactionResponse | TakeOfferResponse | null;
  close: () => void;
  onConfirm?: () => void;
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
  response,
  close,
  onConfirm,
}: ConfirmationDialogProps) {
  const walletState = useWalletState();
  const ticker = walletState.sync.unit.ticker;

  const { addError } = useErrors();

  const [pending, setPending] = useState(false);
  const [signature, setSignature] = useState<string | null>(null);

  useEffect(() => {
    if (response !== null && 'spend_bundle' in response) {
      setSignature(response.spend_bundle.aggregated_signature);
    }
  }, [response]);

  const reset = () => {
    setPending(false);
    setSignature(null);
    close();
  };

  const { created } = response
    ? calculateTransaction(walletState.sync.unit, response.summary)
    : { created: [] };
  const fee = BigNumber(response?.summary.fee || 0);
  const isHighFee = fee.isGreaterThan(1000_000_000); // Adjust threshold as needed

  const json = JSON.stringify(
    response === null
      ? null
      : 'coin_spends' in response
        ? {
            coin_spends: response.coin_spends,
            aggregated_signature:
              signature ||
              '0xc00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000',
          }
        : response.spend_bundle,
    null,
    2,
  ).replace(/"amount": "(.*?)"/g, '"amount": $1');
  // TODO: Fix the above hack

  const [jsonCopied, setJsonCopied] = useState(false);

  const copyJson = () => {
    writeText(json);

    setJsonCopied(true);
    setTimeout(() => setJsonCopied(false), 2000);
  };

  return (
    <Dialog open={!!response} onOpenChange={reset}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            <Trans>Confirm transaction?</Trans>
          </DialogTitle>
          <DialogDescription>
            <div className='max-h-[360px] overflow-y-scroll'>
              <Tabs defaultValue='simple' className='mt-2'>
                <TabsList className='w-full'>
                  <TabsTrigger value='simple' className='flex-grow'>
                    <Trans>Summary</Trans>
                  </TabsTrigger>
                  <TabsTrigger value='advanced' className='flex-grow'>
                    <Trans>Advanced</Trans>
                  </TabsTrigger>
                  <TabsTrigger value='json' className='flex-grow'>
                    <Trans>JSON</Trans>
                  </TabsTrigger>
                </TabsList>
                <TabsContent value='simple'>
                  {isHighFee && !fee.isZero() && (
                    <Alert variant='warning' className='mb-4'>
                      <CircleAlert className='h-4 w-4' />
                      <AlertTitle>
                        <Trans>High Transaction Fee</Trans>
                      </AlertTitle>
                      <AlertDescription>
                        <Trans>
                          Fee exceeds recommended maximum of 0.001 {ticker}
                        </Trans>
                      </AlertDescription>
                    </Alert>
                  )}
                  <Group label={t`Summary`} icon={BadgePlus}>
                    <div className='flex flex-col gap-2'>
                      {!BigNumber(response?.summary.fee || 0).isZero() && (
                        <Item
                          badge={t`Fee`}
                          label={`${toDecimal(response?.summary.fee || '0', walletState.sync.unit.decimals)} ${walletState.sync.unit.ticker}`}
                        />
                      )}
                      {created
                        .filter((created) => created.address !== t`Change`)
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
                  {response !== null && (
                    <AdvancedSummary summary={response.summary} />
                  )}
                </TabsContent>
                <TabsContent value='json'>
                  <Alert>
                    <CircleAlert className='h-4 w-4' />
                    <AlertTitle>
                      <Trans>Warning</Trans>
                    </AlertTitle>
                    <AlertDescription>
                      <Trans>
                        This is the raw JSON spend bundle for this transaction.
                        If you sign it, the transaction can be submitted to the
                        mempool externally.
                      </Trans>
                    </AlertDescription>
                  </Alert>

                  <Button
                    className='mt-2'
                    onClick={() => {
                      commands
                        .signCoinSpends({
                          coin_spends:
                            response === null
                              ? []
                              : 'coin_spends' in response
                                ? response.coin_spends
                                : response.spend_bundle.coin_spends,
                        })
                        .then((data) =>
                          setSignature(data.spend_bundle.aggregated_signature),
                        )
                        .catch(addError);
                    }}
                    disabled={!!signature}
                  >
                    {signature ? (
                      <Trans>Transaction Signed</Trans>
                    ) : (
                      <Trans>Sign Transaction</Trans>
                    )}
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
            <Trans>Cancel</Trans>
          </Button>
          <Button
            onClick={() => {
              setPending(true);

              (async () => {
                let finalSignature: string | null = signature;

                if (
                  !finalSignature &&
                  response !== null &&
                  'coin_spends' in response
                ) {
                  const data = await commands
                    .signCoinSpends({
                      coin_spends: response!.coin_spends,
                    })
                    .catch(addError);

                  if (!data) return reset();

                  finalSignature = data.spend_bundle.aggregated_signature;
                }

                const data = await commands
                  .submitTransaction({
                    spend_bundle: {
                      coin_spends:
                        response === null
                          ? []
                          : 'coin_spends' in response
                            ? response.coin_spends
                            : response.spend_bundle.coin_spends,
                      aggregated_signature: finalSignature!,
                    },
                  })
                  .catch(addError);

                if (!data) return reset();

                onConfirm?.();
                reset();
              })().finally(() => setPending(false));
            }}
            disabled={pending}
          >
            {pending && (
              <LoaderCircleIcon className='mr-2 h-4 w-4 animate-spin' />
            )}
            {pending ? <Trans>Submitting</Trans> : <Trans>Submit</Trans>}
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

export interface AdvancedSummaryProps {
  summary: TransactionSummary;
}

export function AdvancedSummary({ summary }: AdvancedSummaryProps) {
  const walletState = useWalletState();

  const { spent, created } = calculateTransaction(
    walletState.sync.unit,
    summary,
  );

  return (
    <div className='flex flex-col gap-1.5'>
      <Group label={t`Spent Coins`} icon={BadgeMinus}>
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
      <Group label={t`Transaction Output`} icon={BadgePlus}>
        <div className='flex flex-col gap-2'>
          {!BigNumber(summary.fee || 0).isZero() && (
            <div className='flex flex-col gap-1 border-2 p-1.5 rounded-md'>
              <div className='flex items-center gap-2'>
                <Badge>
                  <Trans>Fee</Trans>
                </Badge>
                <span>
                  {toDecimal(summary.fee, walletState.sync.unit.decimals)}{' '}
                  {walletState.sync.unit.ticker}
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
  );
}

interface CalculatedTransaction {
  spent: SpentCoin[];
  created: CreatedCoin[];
}

function calculateTransaction(
  xch: Unit,
  summary: TransactionSummary,
): CalculatedTransaction {
  const spent: SpentCoin[] = [];
  const created: CreatedCoin[] = [];

  for (const input of summary.inputs || []) {
    if (input.type === 'xch') {
      spent.push({
        badge: 'Chia',
        label: `${toDecimal(input.amount, xch.decimals)} ${xch.ticker}`,
        coinId: input.coin_id,
        sort: 1,
      });

      for (const output of input.outputs) {
        if (summary.inputs.find((i) => i.coin_id === output.coin_id)) {
          continue;
        }

        created.push({
          badge: 'Chia',
          label: `${toDecimal(output.amount, xch.decimals)} ${xch.ticker}`,
          address: output.burning
            ? t`Permanently Burned`
            : output.receiving
              ? t`You`
              : output.address,
          sort: 1,
        });
      }
    }

    if (input.type === 'cat') {
      const ticker = input.ticker || 'CAT';

      spent.push({
        badge: `CAT ${input.name || input.asset_id}`,
        label: `${toDecimal(input.amount, 3)} ${ticker}`,
        coinId: input.coin_id,
        sort: 2,
      });

      for (const output of input.outputs) {
        if (summary.inputs.find((i) => i.coin_id === output.coin_id)) {
          continue;
        }

        created.push({
          badge: `CAT ${input.name || input.asset_id}`,
          label: `${toDecimal(output.amount, 3)} ${ticker}`,
          address: output.burning
            ? t`Permanently Burned`
            : output.receiving
              ? t`You`
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
          badge: t`Profile`,
          label: input.name || t`Unnamed`,
          coinId: input.coin_id,
          sort: 3,
        });
      }

      for (const output of input.outputs) {
        if (summary.inputs.find((i) => i.coin_id === output.coin_id)) {
          continue;
        }

        if (BigNumber(output.amount).mod(2).eq(1)) {
          created.push({
            badge: t`Profile`,
            label: input.name || t`Unnamed`,
            address: output.burning
              ? t`Permanently Burned`
              : output.receiving
                ? t`You`
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
          label: input.name || t`Unknown`,
          coinId: input.coin_id,
          sort: 4,
        });
      }

      for (const output of input.outputs) {
        if (summary.inputs.find((i) => i.coin_id === output.coin_id)) {
          continue;
        }

        if (BigNumber(output.amount).mod(2).isEqualTo(1)) {
          created.push({
            badge: 'NFT',
            label: input.name || t`Unknown`,
            address: output.burning
              ? t`Permanently Burned`
              : output.receiving
                ? t`You`
                : output.address,
            sort: 4,
          });
        }
      }
    }
  }

  return { spent, created };
}
