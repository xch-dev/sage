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
import { fromMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import BigNumber from 'bignumber.js';
import {
  BadgeMinus,
  BadgePlus,
  BoxIcon,
  BracesIcon,
  CopyCheckIcon,
  CopyIcon,
  ForwardIcon,
  LoaderCircleIcon,
  ArrowUpIcon,
  InfoIcon,
  AlertCircleIcon,
  CheckCircleIcon,
  ListCollapseIcon,
  ArrowUpRightIcon,
} from 'lucide-react';
import { useEffect, useState } from 'react';
import { commands, TakeOfferResponse, TransactionResponse } from '../bindings';
import { Alert, AlertDescription, AlertTitle } from './ui/alert';
import { Badge } from './ui/badge';
import { Separator } from './ui/separator';
import { CopyButton } from './CopyButton';
import { toast } from 'react-toastify';
import { Tabs, TabsContent, TabsList, TabsTrigger } from './ui/tabs';
import { formatNumber } from '../i18n';
import { calculateTransaction } from './AdvancedTransactionSummary';

export interface ConfirmationDialogProps {
  response: TransactionResponse | TakeOfferResponse | null;
  close: () => void;
  onConfirm?: () => void;
  additionalData?: {
    title: string;
    content: React.ReactNode;
  };
  showRecipientDetails?: boolean;
}

export default function ConfirmationDialog({
  response,
  close,
  onConfirm,
  additionalData,
  showRecipientDetails = true,
}: ConfirmationDialogProps) {
  const walletState = useWalletState();
  const ticker = walletState.sync.unit.ticker;

  const { addError } = useErrors();

  const [pending, setPending] = useState(false);
  const [signature, setSignature] = useState<string | null>(null);
  const [jsonCopied, setJsonCopied] = useState(false);
  const [activeTab, setActiveTab] = useState<string>('summary');

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

  const { spent, created } = response
    ? calculateTransaction(walletState.sync.unit, response.summary)
    : { spent: [], created: [] };
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

  const copyJson = () => {
    writeText(json);
    setJsonCopied(true);
    setTimeout(() => setJsonCopied(false), 2000);
    toast.success(t`JSON copied to clipboard`);
  };

  // Group assets by type for combined display
  const groupedAssets = created
    .filter((item) => item.address !== t`Change` && item.address !== t`You`)
    .sort((a, b) => a.sort - b.sort)
    .reduce(
      (acc, item) => {
        // Extract amount and ticker from the label (e.g., "1.234 XCH")
        const labelParts = item.label.split(' ');
        const amount = labelParts[0];
        const ticker = labelParts.length > 1 ? labelParts[1] : '';

        // Group by badge and ticker to combine identical assets
        const key = `${item.badge}-${ticker}`;
        if (!acc[key]) {
          acc[key] = {
            badge: item.badge,
            label: item.label,
            originalLabel: item.label, // Store original label
            ticker: ticker,
            recipients: [item.address],
            amounts: [amount],
            totalAmount: amount,
          };
        } else {
          // Add recipient if not already in the list
          if (!acc[key].recipients.includes(item.address)) {
            acc[key].recipients.push(item.address);
            acc[key].amounts.push(amount);

            // Update total amount
            try {
              const currentTotal = parseFloat(
                acc[key].totalAmount.replace(/,/g, ''),
              );
              const newAmount = parseFloat(amount.replace(/,/g, ''));
              const newTotal = currentTotal + newAmount;
              acc[key].totalAmount = formatNumber({
                value: newTotal,
                minimumFractionDigits: 0,
                maximumFractionDigits: 12,
              });
              // Keep the original label format but update the total
              acc[key].label = acc[key].originalLabel;
            } catch (e) {
              // If parsing fails, keep the original amount
            }
          }
        }
        return acc;
      },
      {} as Record<
        string,
        {
          badge: string;
          label: string;
          originalLabel: string;
          ticker: string;
          recipients: string[];
          amounts: string[];
          totalAmount: string;
        }
      >,
    );

  return (
    <Dialog open={!!response} onOpenChange={reset}>
      <DialogContent className='max-w-none w-full h-full md:max-w-[500px] md:h-[80vh] flex flex-col p-2.5 md:p-6 border-0 md:border rounded-none md:rounded-lg'>
        <DialogHeader className='flex-shrink-0 mt-12 sm:mt-0'>
          <DialogTitle className='text-xl font-semibold'>
            <Trans>Confirm Transaction</Trans>
          </DialogTitle>
          <DialogDescription>
            <div className='text-sm text-muted-foreground'>
              <Trans>
                Please review the transaction details before submitting.
              </Trans>
            </div>
          </DialogDescription>
        </DialogHeader>

        <div className='flex-1 overflow-hidden flex flex-col'>
          <Tabs
            value={activeTab}
            onValueChange={setActiveTab}
            className='w-full h-full flex flex-col'
          >
            <div className='flex items-center justify-between mb-4 flex-shrink-0'>
              <TabsList className='w-full inline-flex h-9 items-center justify-start rounded-lg bg-muted p-1 text-muted-foreground'>
                <TabsTrigger
                  value='summary'
                  className='flex-1 rounded-md px-3 py-1 text-sm font-medium'
                >
                  <div className='flex items-center justify-center'>
                    <InfoIcon className='h-4 w-4 mr-2' />
                    <Trans>Summary</Trans>
                  </div>
                </TabsTrigger>

                <TabsTrigger
                  value='details'
                  className='flex-1 rounded-md px-3 py-1 text-sm font-medium'
                >
                  <div className='flex items-center justify-center'>
                    <ListCollapseIcon className='h-4 w-4 mr-2' />
                    <Trans>Details</Trans>
                  </div>
                </TabsTrigger>

                <TabsTrigger
                  value='json'
                  className='flex-1 rounded-md px-3 py-1 text-sm font-medium'
                >
                  <div className='flex items-center justify-center'>
                    <BracesIcon className='h-4 w-4 mr-2' />
                    <Trans>JSON</Trans>
                  </div>
                </TabsTrigger>
              </TabsList>
            </div>

            <div className='flex-1 relative'>
              {/* Transaction Summary Tab */}
              <TabsContent
                value='summary'
                className='absolute inset-0 overflow-auto border rounded-md p-4 bg-white dark:bg-neutral-950'
              >
                {/* High Fee Warning */}
                {isHighFee && !fee.isZero() && (
                  <Alert variant='warning' className='mb-3'>
                    <AlertCircleIcon className='h-4 w-4' />
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

                {/* Fee Display */}

                {/* Additional Data Display */}
                {additionalData && (
                  <div className='mb-4'>
                    <h3 className='text-sm font-medium mb-2 flex items-center'>
                      <InfoIcon className='h-4 w-4 mr-1' />
                      {additionalData.title}
                    </h3>
                    <div className='space-y-2'>
                      <div className='flex items-start gap-2 text-sm border rounded-md p-2 bg-neutral-50 dark:bg-neutral-900'>
                        <div className='break-words whitespace-pre-wrap w-full'>
                          {additionalData.content}
                        </div>
                      </div>
                    </div>
                  </div>
                )}

                {/* Combined Assets and Recipients */}
                {showRecipientDetails && (
                  <div>
                    <h3 className='text-sm font-medium mb-2 flex items-center'>
                      <ArrowUpIcon className='h-4 w-4 mr-1' />
                      <Trans>Sending</Trans>
                    </h3>
                    <div className='space-y-4'>
                      {Object.entries(groupedAssets).map(([key, group]) => (
                        <div
                          key={key}
                          className='flex flex-col gap-1.5 rounded-md border p-3'
                        >
                          <div className='flex items-center justify-between'>
                            <Badge className='max-w-[100px]'>
                              <span className='truncate'>{group.badge}</span>
                            </Badge>
                            <div className='flex items-center'>
                              <ArrowUpRightIcon className='h-4 w-4 mr-1 text-blue-500' />
                              <Trans>Total:</Trans>{' '}
                              <span className='font-medium text-foreground ml-1'>
                                {group.totalAmount} {group.ticker}
                              </span>
                            </div>
                          </div>

                          <Separator className='my-2' />

                          {/* Show recipients */}
                          <div className='flex flex-col gap-2'>
                            <div className='text-sm font-medium text-muted-foreground flex items-center justify-between'>
                              <div className='flex items-center'>
                                <Trans>To:</Trans>{' '}
                                {group.recipients.length > 1 && (
                                  <span className='ml-1 text-xs bg-neutral-100 dark:bg-neutral-800 px-1.5 py-0.5 rounded-full'>
                                    <Trans id='sending_to_recipients'>
                                      Sending <span>{group.label}</span> to{' '}
                                      <span>{group.recipients.length}</span>{' '}
                                      recipients
                                    </Trans>
                                  </span>
                                )}
                              </div>
                            </div>

                            <div
                              className={
                                group.recipients.length > 3
                                  ? 'max-h-[150px] overflow-y-auto pr-1'
                                  : ''
                              }
                            >
                              {group.recipients.map((address, i) => (
                                <div
                                  key={i}
                                  className='flex items-center gap-1.5 min-w-0 w-full pl-2'
                                >
                                  <ForwardIcon className='w-4 h-4 text-blue-500 shrink-0' />
                                  <div className='text-sm truncate flex-1'>
                                    {address}
                                  </div>
                                  {address !== t`Permanently Burned` &&
                                    address !== t`You` &&
                                    address !== t`Change` && (
                                      <CopyButton
                                        value={address}
                                        className='h-4 w-4 shrink-0'
                                        onCopy={() =>
                                          toast.success(
                                            t`Address copied to clipboard`,
                                          )
                                        }
                                      />
                                    )}
                                </div>
                              ))}
                            </div>
                          </div>
                        </div>
                      ))}

                      {created.filter(
                        (item) =>
                          item.address !== t`Change` && item.address !== t`You`,
                      ).length === 0 && (
                        <div className='text-sm text-muted-foreground p-3 border rounded-md'>
                          <Trans>
                            No assets being sent to external recipients.
                          </Trans>
                        </div>
                      )}
                    </div>
                  </div>
                )}
              </TabsContent>

              {/* Transaction Details Tab */}
              <TabsContent
                value='details'
                className='absolute inset-0 overflow-auto border rounded-md p-4 bg-white dark:bg-neutral-950'
              >
                <div className='flex flex-col gap-4'>
                  {/* Spent Coins */}
                  <div>
                    <h3 className='text-sm font-semibold mb-2 flex items-center'>
                      <BadgeMinus className='h-4 w-4 mr-1' />
                      <Trans>Spent Coins</Trans>
                    </h3>
                    <div className='space-y-2'>
                      {spent.length === 0 ? (
                        <div className='text-sm text-muted-foreground p-3 border rounded-md'>
                          <Trans>No coins being spent.</Trans>
                        </div>
                      ) : (
                        spent
                          .sort((a, b) => a.sort - b.sort)
                          .map((item, i) => (
                            <div
                              key={i}
                              className='flex flex-col gap-1.5 rounded-md border p-2'
                            >
                              <div className='flex items-center justify-between'>
                                <Badge className='shrink-0'>{item.badge}</Badge>
                                <div className='font-medium'>{item.label}</div>
                              </div>
                              <div className='flex items-center gap-1.5'>
                                <BoxIcon className='w-4 h-4 shrink-0 text-muted-foreground' />
                                <div className='text-xs text-muted-foreground truncate font-mono flex-1'>
                                  {item.coinId.slice(0, 6) +
                                    '...' +
                                    item.coinId.slice(-6)}
                                </div>
                                <CopyButton
                                  value={item.coinId}
                                  className='h-4 w-4 shrink-0'
                                  onCopy={() =>
                                    toast.success(
                                      t`Coin ID copied to clipboard`,
                                    )
                                  }
                                />
                              </div>
                            </div>
                          ))
                      )}
                    </div>
                  </div>

                  {/* Transaction Output */}
                  <div>
                    <h3 className='text-sm font-semibold mb-2 flex items-center'>
                      <BadgePlus className='h-4 w-4 mr-1' />
                      <Trans>Transaction Output</Trans>
                    </h3>
                    <div className='space-y-2'>
                      {/* Fee */}
                      {!fee.isZero() && (
                        <div className='flex flex-col gap-1.5 rounded-md border p-2'>
                          <div className='flex items-center justify-between'>
                            <Badge
                              variant='outline'
                              className='bg-amber-100 dark:bg-amber-950 border-amber-200 dark:border-amber-800 text-amber-800 dark:text-amber-300'
                            >
                              <Trans>Fee</Trans>
                            </Badge>
                            <span className='text-sm font-medium'>
                              {formatNumber({
                                value: fromMojos(
                                  fee,
                                  walletState.sync.unit.decimals,
                                ),
                                minimumFractionDigits: 0,
                                maximumFractionDigits:
                                  walletState.sync.unit.decimals,
                              })}{' '}
                              {ticker}
                            </span>
                          </div>
                        </div>
                      )}

                      {/* Created Coins */}
                      {created
                        .sort((a, b) => a.sort - b.sort)
                        .map((item, i) => (
                          <div
                            key={i}
                            className='flex flex-col gap-1.5 rounded-md border p-2'
                          >
                            <div className='flex items-center justify-between'>
                              <Badge className='shrink-0'>{item.badge}</Badge>
                              <div className='font-medium'>{item.label}</div>
                            </div>
                            <div className='flex items-center gap-1.5'>
                              <ForwardIcon className='w-4 h-4 shrink-0 text-blue-500' />
                              <div className='text-sm truncate flex-1'>
                                {item.address}
                              </div>
                              {item.address !== t`Permanently Burned` &&
                                item.address !== t`You` &&
                                item.address !== t`Change` && (
                                  <CopyButton
                                    value={item.address}
                                    className='h-4 w-4 shrink-0'
                                    onCopy={() =>
                                      toast.success(
                                        t`Address copied to clipboard`,
                                      )
                                    }
                                  />
                                )}
                            </div>
                          </div>
                        ))}
                    </div>
                  </div>
                </div>
              </TabsContent>

              {/* JSON Tab */}
              <TabsContent
                value='json'
                className='absolute inset-0 overflow-auto border rounded-md p-4 bg-white dark:bg-neutral-950'
              >
                <Alert variant='warning'>
                  <AlertCircleIcon className='h-4 w-4' />
                  <AlertTitle>
                    <Trans>Advanced Feature</Trans>
                  </AlertTitle>
                  <AlertDescription>
                    <Trans>
                      This is the raw JSON spend bundle for this transaction. If
                      you sign it, the transaction can be submitted to the
                      mempool externally.
                    </Trans>
                  </AlertDescription>
                </Alert>

                <div className='flex items-center gap-2 mt-4'>
                  <Button
                    size='sm'
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
                        .then((data) => {
                          setSignature(data.spend_bundle.aggregated_signature);
                          toast.success(t`Transaction signed successfully`);
                        })
                        .catch(addError);
                    }}
                    disabled={!!signature}
                  >
                    {signature ? (
                      <>
                        <CheckCircleIcon className='h-4 w-4 mr-1' />
                        <Trans>Signed</Trans>
                      </>
                    ) : (
                      <Trans>Sign Transaction</Trans>
                    )}
                  </Button>

                  <Button
                    variant='outline'
                    size='sm'
                    onClick={copyJson}
                    className='flex items-center gap-1'
                  >
                    {jsonCopied ? (
                      <>
                        <CopyCheckIcon className='h-4 w-4 text-emerald-500' />
                        <Trans>Copied</Trans>
                      </>
                    ) : (
                      <>
                        <CopyIcon className='h-4 w-4' />
                        <Trans>Copy JSON</Trans>
                      </>
                    )}
                  </Button>
                </div>

                <div className='relative p-3 mt-4 break-all rounded-md bg-neutral-100 dark:bg-neutral-900 whitespace-pre-wrap text-xs font-mono'>
                  {json}
                </div>
              </TabsContent>
            </div>
          </Tabs>
        </div>

        <div className='grid grid-cols-1 gap-4 mb-2'>
          <div className='flex items-center justify-between text-sm text-muted-foreground px-2'>
            <span>
              <Trans>Transaction Fee</Trans>
            </span>
            <span>
              {fee.isZero() ? (
                '-'
              ) : (
                <>
                  {formatNumber({
                    value: fromMojos(fee, walletState.sync.unit.decimals),
                    minimumFractionDigits: 0,
                    maximumFractionDigits: walletState.sync.unit.decimals,
                  })}{' '}
                  {ticker}
                </>
              )}
            </span>
          </div>
        </div>

        <DialogFooter className='pt-4 flex-shrink-0 border-t mt-auto mb-6 sm:mb-0 mr-4 sm:mr-0'>
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

                toast.success(t`Transaction submitted successfully`);
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
