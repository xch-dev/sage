import {
  Amount,
  Asset,
  commands,
  events,
  OptionRecord,
  TransactionResponse,
} from '@/bindings';
import { AssetIcon } from '@/components/AssetIcon';
import ConfirmationDialog from '@/components/ConfirmationDialog';
import { OptionConfirmation } from '@/components/confirmations/OptionConfirmation';
import Container from '@/components/Container';
import { FeeOnlyDialog } from '@/components/FeeOnlyDialog';
import Header from '@/components/Header';
import { NumberFormat } from '@/components/NumberFormat';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { TransferDialog } from '@/components/TransferDialog';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
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
import { Switch } from '@/components/ui/switch';
import { CustomError } from '@/contexts/ErrorContext';
import { useErrors } from '@/hooks/useErrors';
import { fromMojos, toMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Plural, Trans } from '@lingui/react/macro';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import {
  Copy,
  EyeIcon,
  EyeOff,
  FilePenLine,
  Flame,
  HandCoins,
  MoreVerticalIcon,
  SendIcon,
} from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { toast } from 'react-toastify';

export function OptionList() {
  const navigate = useNavigate();

  const { addError } = useErrors();

  const [options, setOptions] = useState<OptionRecord[]>([]);

  const updateOptions = useCallback(async () => {
    try {
      const data = await commands.getOptions({});
      setOptions(data.options);
    } catch (error) {
      addError(error as CustomError);
    }
  }, [addError]);

  useEffect(() => {
    updateOptions();

    const unlisten = events.syncEvent.listen((event) => {
      const type = event.payload.type;

      if (type === 'coin_state' || type === 'puzzle_batch_synced') {
        updateOptions();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [updateOptions]);

  const [showHidden, setShowHidden] = useState(false);

  const visibleOptions = showHidden
    ? options
    : options.filter((option) => option.visible);
  const hasHiddenOptions = options.findIndex((option) => !option.visible) > -1;

  return (
    <>
      <Header title={t`Option Contracts`}>
        <ReceiveAddress />
      </Header>
      <Container>
        <Button onClick={() => navigate('/options/mint')}>
          <FilePenLine className='h-4 w-4 mr-2' />
          <Trans>Mint Option</Trans>
        </Button>

        {hasHiddenOptions && (
          <div className='flex items-center gap-2 my-4'>
            <label htmlFor='viewHidden'>
              <Trans>View hidden</Trans>
            </label>
            <Switch
              id='viewHidden'
              checked={showHidden}
              onCheckedChange={(value) => setShowHidden(value)}
            />
          </div>
        )}

        {options.length === 0 && (
          <Alert className='mt-4'>
            <FilePenLine className='h-4 w-4' />
            <AlertTitle>
              <Trans>Create an option?</Trans>
            </AlertTitle>
            <AlertDescription>
              <Plural
                value={options.length}
                one='You do not currently have any option contracts. Would you like to mint one?'
                other='You do not currently have any option contracts. Would you like to mint one?'
              />
            </AlertDescription>
          </Alert>
        )}

        <div className='mt-4 grid gap-4 md:gap-4 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4'>
          {visibleOptions.map((option) => (
            <Option
              key={option.launcher_id}
              option={option}
              updateOptions={updateOptions}
            />
          ))}
        </div>
      </Container>
    </>
  );
}

interface OptionProps {
  option: OptionRecord;
  updateOptions: () => void;
}

function Option({ option, updateOptions }: OptionProps) {
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
            {option.underlying_asset.ticker ?? 'CAT'} /{' '}
            {option.strike_asset.ticker ?? 'CAT'}
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
          <div className='text-sm font-medium truncate mb-2'>
            {option.launcher_id}
          </div>

          <OptionAssetPreview
            asset={option.underlying_asset}
            amount={option.underlying_amount}
          />
          <OptionAssetPreview
            asset={option.strike_asset}
            amount={option.strike_amount}
          />
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
}

function OptionAssetPreview({ asset, amount }: OptionAssetPreviewProps) {
  return (
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
  );
}
