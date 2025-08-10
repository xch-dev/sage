import {
  Amount,
  Asset,
  commands,
  OptionRecord,
  TransactionResponse,
} from '@/bindings';
import { AssetIcon } from '@/components/AssetIcon';
import ConfirmationDialog from '@/components/ConfirmationDialog';
import { OptionConfirmation } from '@/components/confirmations/OptionConfirmation';
import { CopyBox } from '@/components/CopyBox';
import { FeeOnlyDialog } from '@/components/FeeOnlyDialog';
import { NumberFormat } from '@/components/NumberFormat';
import { TransferDialog } from '@/components/TransferDialog';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Separator } from '@/components/ui/separator';
import { useErrors } from '@/hooks/useErrors';
import { formatTimestamp, fromMojos, toMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { InfoCircledIcon } from '@radix-ui/react-icons';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import {
  AlertCircle,
  Copy,
  EyeIcon,
  EyeOff,
  FilePenLine,
  Flame,
  HandCoins,
  MoreVerticalIcon,
  SendIcon,
} from 'lucide-react';
import { useState } from 'react';
import { toast } from 'react-toastify';

interface OptionGridViewProps {
  options: OptionRecord[];
  updateOptions: () => void;
  showHidden: boolean;
}

export function OptionGridView({
  options,
  updateOptions,
  showHidden,
}: OptionGridViewProps) {
  const visibleOptions = showHidden
    ? options
    : options.filter((option) => option.visible);

  return (
    <div className='mt-4 grid gap-4 md:gap-4 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4'>
      {visibleOptions.map((option) => (
        <OptionCard
          key={option.launcher_id}
          option={option}
          updateOptions={updateOptions}
        />
      ))}
    </div>
  );
}

interface OptionCardProps {
  option: OptionRecord;
  updateOptions: () => void;
}

function OptionCard({ option, updateOptions }: OptionCardProps) {
  const { addError } = useErrors();

  const walletState = useWalletState();

  const [response, setResponse] = useState<TransactionResponse | null>(null);

  const [exerciseOpen, setExerciseOpen] = useState(false);
  const [transferOpen, setTransferOpen] = useState(false);
  const [burnOpen, setBurnOpen] = useState(false);

  const [isExercising, setIsExercising] = useState(false);
  const [isTransferring, setIsTransferring] = useState(false);
  const [isBurning, setIsBurning] = useState(false);
  const [transferAddress, setTransferAddress] = useState('');

  const onExerciseSubmit = (fee: string) => {
    setIsExercising(true);
    commands
      .exerciseOptions({
        option_ids: [option.launcher_id],
        fee: toMojos(fee, walletState.sync.unit.precision),
      })
      .then(setResponse)
      .catch((err) => {
        setIsExercising(false);
        addError(err);
      })
      .finally(() => setExerciseOpen(false));
  };

  const onTransferSubmit = (address: string, fee: string) => {
    setIsTransferring(true);
    setTransferAddress(address);
    commands
      .transferOptions({
        option_ids: [option.launcher_id],
        address,
        fee: toMojos(fee, walletState.sync.unit.precision),
      })
      .then(setResponse)
      .catch((err) => {
        setIsTransferring(false);
        addError(err);
      })
      .finally(() => setTransferOpen(false));
  };

  const onBurnSubmit = (fee: string) => {
    setIsBurning(true);
    commands
      .transferOptions({
        option_ids: [option.launcher_id],
        address: walletState.sync.burn_address,
        fee: toMojos(fee, walletState.sync.unit.precision),
      })
      .then(setResponse)
      .catch((err) => {
        setIsBurning(false);
        addError(err);
      })
      .finally(() => setBurnOpen(false));
  };

  return (
    <>
      <Card
        key={option.launcher_id}
        className={`${!option.visible ? 'opacity-50 grayscale' : option.created_height === null ? 'pulsate-opacity' : ''}`}
      >
        <CardHeader className='-mt-2 flex flex-row items-center justify-between space-y-0 pb-2 pr-2 space-x-2'>
          <CardTitle className='text-md font-medium truncate flex items-center'>
            <FilePenLine className='mr-2 h-4 w-4' />
            {option.name}
          </CardTitle>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant='ghost' size='icon'>
                <MoreVerticalIcon className='h-5 w-5' />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align='end'>
              <DropdownMenuGroup>
                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    setExerciseOpen(true);
                  }}
                  disabled={option.created_height === null}
                >
                  <HandCoins className='mr-2 h-4 w-4' />
                  <span>
                    <Trans>Exercise</Trans>
                  </span>
                </DropdownMenuItem>

                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    setTransferOpen(true);
                  }}
                  disabled={option.created_height === null}
                >
                  <SendIcon className='mr-2 h-4 w-4' />
                  <span>
                    <Trans>Transfer</Trans>
                  </span>
                </DropdownMenuItem>

                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    setBurnOpen(true);
                  }}
                  disabled={option.created_height === null}
                >
                  <Flame className='mr-2 h-4 w-4' />
                  <span>
                    <Trans>Burn</Trans>
                  </span>
                </DropdownMenuItem>

                <DropdownMenuSeparator />

                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    writeText(option.launcher_id);
                    toast.success(t`Option ID copied to clipboard`);
                  }}
                >
                  <Copy className='mr-2 h-4 w-4' />
                  <span>
                    <Trans>Copy ID</Trans>
                  </span>
                </DropdownMenuItem>

                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    // toggleVisibility();
                  }}
                >
                  {option.visible ? (
                    <EyeOff className='mr-2 h-4 w-4' />
                  ) : (
                    <EyeIcon className='mr-2 h-4 w-4' />
                  )}
                  <span>{option.visible ? t`Hide` : t`Show`}</span>
                </DropdownMenuItem>
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>
        </CardHeader>
        <CardContent>
          <div className='flex flex-col gap-2'>
            <div className='flex flex-col gap-1'>
              <div className='text-sm font-medium text-muted-foreground'>
                Option ID
              </div>
              <CopyBox value={option.launcher_id} title={t`Option ID`} />
            </div>

            <div className='flex flex-col gap-1'>
              <div className='text-sm font-medium text-muted-foreground'>
                Expiration
              </div>
              <div className='text-sm font-medium'>
                {formatTimestamp(option.expiration_seconds)}
              </div>
            </div>

            {option.expiration_seconds * 1000 < Date.now() ? (
              <div className='flex items-center gap-1.5 text-sm font-medium text-red-500'>
                <AlertCircle className='h-4 w-4' />
                <Trans>Expired</Trans>
              </div>
            ) : option.expiration_seconds * 1000 <
              Date.now() + 24 * 60 * 60 * 1000 ? (
              <div className='flex items-center gap-1.5 text-sm font-medium text-yellow-500'>
                <AlertCircle className='h-4 w-4' />
                <Trans>Expiring soon</Trans>
              </div>
            ) : (
              <div className='flex items-center gap-1.5 text-sm font-medium text-blue-400'>
                <InfoCircledIcon className='h-4 w-4' />
                <Trans>Active</Trans>
              </div>
            )}

            <Separator className='my-1' />

            <div className='flex flex-col gap-2'>
              <OptionAssetPreview
                asset={option.underlying_asset}
                amount={option.underlying_amount}
                option={option}
              />
              <OptionAssetPreview
                asset={option.strike_asset}
                amount={option.strike_amount}
                option={option}
              />
            </div>
          </div>
        </CardContent>
      </Card>

      <FeeOnlyDialog
        title={t`Exercise Option`}
        submitButtonLabel={t`Exercise`}
        open={exerciseOpen}
        setOpen={setExerciseOpen}
        onSubmit={onExerciseSubmit}
      >
        <Trans>
          This will exercise the option contract by paying its strike price and
          unlocking the underlying asset.
        </Trans>
      </FeeOnlyDialog>

      <TransferDialog
        title={t`Transfer Option`}
        open={transferOpen}
        setOpen={setTransferOpen}
        onSubmit={onTransferSubmit}
      >
        <Trans>This will send the option to the provided address.</Trans>
      </TransferDialog>

      <FeeOnlyDialog
        title={t`Burn Option`}
        submitButtonLabel={t`Burn`}
        open={burnOpen}
        setOpen={setBurnOpen}
        onSubmit={onBurnSubmit}
      >
        <Trans>
          This will permanently delete the option by sending it to the burn
          address.
        </Trans>
      </FeeOnlyDialog>

      <ConfirmationDialog
        response={response}
        showRecipientDetails={false}
        close={() => {
          setResponse(null);
          setIsTransferring(false);
          setIsBurning(false);
        }}
        onConfirm={() => updateOptions()}
        additionalData={
          isTransferring && response
            ? {
                title: t`Transfer Option`,
                content: (
                  <OptionConfirmation
                    options={[option]}
                    address={transferAddress}
                    type='transfer'
                  />
                ),
              }
            : isBurning && response
              ? {
                  title: t`Burn Option`,
                  content: (
                    <OptionConfirmation options={[option]} type='burn' />
                  ),
                }
              : isExercising && response
                ? {
                    title: t`Exercise Option`,
                    content: (
                      <OptionConfirmation options={[option]} type='exercise' />
                    ),
                  }
                : undefined
        }
      />
    </>
  );
}

interface OptionAssetPreviewProps {
  asset: Asset;
  amount: Amount;
  option: OptionRecord;
}

function OptionAssetPreview({
  asset,
  amount,
  option,
}: OptionAssetPreviewProps) {
  return (
    <div className='flex flex-col gap-1'>
      <div className='text-sm font-medium text-muted-foreground'>
        {asset === option.underlying_asset
          ? 'Underlying Asset'
          : 'Strike Price'}
      </div>
      <div className='flex items-center gap-2' key={asset.asset_id ?? 'xch'}>
        <AssetIcon asset={asset} size='md' />
        <div className='text-sm text-muted-foreground truncate'>
          {asset.kind === 'token' && (
            <NumberFormat
              value={fromMojos(amount, asset.precision)}
              minimumFractionDigits={0}
              maximumFractionDigits={asset.precision}
            />
          )}{' '}
          {asset.name ?? asset.ticker ?? t`Unknown`}
        </div>
      </div>
    </div>
  );
}
