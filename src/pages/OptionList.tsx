import {
  commands,
  events,
  OptionRecord,
  TransactionResponse,
} from '@/bindings';
import ConfirmationDialog from '@/components/ConfirmationDialog';
import { OptionConfirmation } from '@/components/confirmations/OptionConfirmation';
import Container from '@/components/Container';
import { FeeOnlyDialog } from '@/components/FeeOnlyDialog';
import Header from '@/components/Header';
import { OptionActionHandlers } from '@/components/OptionColumns';
import { OptionGridView } from '@/components/OptionGridView';
import { OptionListView } from '@/components/OptionListView';
import { OptionOptions } from '@/components/OptionOptions';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { TransferDialog } from '@/components/TransferDialog';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { CustomError } from '@/contexts/ErrorContext';
import { useErrors } from '@/hooks/useErrors';
import { useOptionParams } from '@/hooks/useOptionParams';
import { toMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Plural, Trans } from '@lingui/react/macro';
import { FilePenLine } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';

export function OptionList() {
  const navigate = useNavigate();
  const { addError } = useErrors();
  const walletState = useWalletState();
  const [params, setParams] = useOptionParams();
  const {
    viewMode,
    sortMode,
    ascending,
    showHiddenOptions,
    search,
    page,
    limit,
  } = params;
  const [options, setOptions] = useState<OptionRecord[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(false);

  // Dialog states for list view actions
  const [selectedOption, setSelectedOption] = useState<OptionRecord | null>(
    null,
  );
  const [response, setResponse] = useState<TransactionResponse | null>(null);
  const [exerciseOpen, setExerciseOpen] = useState(false);
  const [transferOpen, setTransferOpen] = useState(false);
  const [burnOpen, setBurnOpen] = useState(false);
  const [isExercising, setIsExercising] = useState(false);
  const [isTransferring, setIsTransferring] = useState(false);
  const [isBurning, setIsBurning] = useState(false);
  const [transferAddress, setTransferAddress] = useState('');

  const updateOptions = useCallback(async () => {
    setLoading(true);
    try {
      const offset = (page - 1) * limit;
      const data = await commands.getOptions({
        offset,
        limit,
        sort_mode: sortMode,
        ascending,
        find_value: search || null,
        include_hidden: showHiddenOptions,
      });

      setOptions(data.options);
      setTotal(data.total);
    } catch (error) {
      addError(error as CustomError);
    } finally {
      setLoading(false);
    }
  }, [addError, page, limit, sortMode, ascending, search, showHiddenOptions]);

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

  // Action handlers for list view
  const onExerciseSubmit = useCallback(
    (fee: string) => {
      if (!selectedOption) return;
      setIsExercising(true);
      commands
        .exerciseOptions({
          option_ids: [selectedOption.launcher_id],
          fee: toMojos(fee, walletState.sync.unit.precision),
        })
        .then(setResponse)
        .catch((err) => {
          setIsExercising(false);
          addError(err);
        })
        .finally(() => setExerciseOpen(false));
    },
    [selectedOption, walletState.sync.unit.precision, addError],
  );

  const onTransferSubmit = useCallback(
    (address: string, fee: string) => {
      if (!selectedOption) return;
      setIsTransferring(true);
      setTransferAddress(address);
      commands
        .transferOptions({
          option_ids: [selectedOption.launcher_id],
          address,
          fee: toMojos(fee, walletState.sync.unit.precision),
        })
        .then(setResponse)
        .catch((err) => {
          setIsTransferring(false);
          addError(err);
        })
        .finally(() => setTransferOpen(false));
    },
    [selectedOption, walletState.sync.unit.precision, addError],
  );

  const onBurnSubmit = useCallback(
    (fee: string) => {
      if (!selectedOption) return;
      setIsBurning(true);
      commands
        .transferOptions({
          option_ids: [selectedOption.launcher_id],
          address: walletState.sync.burn_address,
          fee: toMojos(fee, walletState.sync.unit.precision),
        })
        .then(setResponse)
        .catch((err) => {
          setIsBurning(false);
          addError(err);
        })
        .finally(() => setBurnOpen(false));
    },
    [
      selectedOption,
      walletState.sync.unit.precision,
      walletState.sync.burn_address,
      addError,
    ],
  );

  const optionActionHandlers: OptionActionHandlers = {
    onExercise: (option) => {
      setSelectedOption(option);
      setExerciseOpen(true);
    },
    onTransfer: (option) => {
      setSelectedOption(option);
      setTransferOpen(true);
    },
    onBurn: (option) => {
      setSelectedOption(option);
      setBurnOpen(true);
    },
    onToggleVisibility: (option) => {
      commands
        .updateOption({
          option_id: option.launcher_id,
          visible: !option.visible,
        })
        .then(updateOptions)
        .catch(addError);
    },
  };

  return (
    <>
      <Header title={t`Option Contracts`}>
        <div className='flex items-center gap-2'>
          <ReceiveAddress />
        </div>
      </Header>
      <Container>
        <Button
          aria-label={t`Mint new option`}
          className='mb-4'
          onClick={() => navigate('/options/mint')}
        >
          <FilePenLine className='h-4 w-4 mr-2' />
          <Trans>Mint Option</Trans>
        </Button>

        <OptionOptions
          query={search}
          setQuery={(value) => setParams({ search: value, page: 1 })}
          viewMode={viewMode}
          setViewMode={(value) => setParams({ viewMode: value })}
          sortMode={sortMode}
          setSortMode={(value) => setParams({ sortMode: value, page: 1 })}
          ascending={ascending}
          setAscending={(value) => setParams({ ascending: value, page: 1 })}
          showHiddenOptions={showHiddenOptions}
          setShowHiddenOptions={(value) =>
            setParams({ showHiddenOptions: value, page: 1 })
          }
          handleSearch={(value) => {
            setParams({ search: value, page: 1 });
          }}
          className='mb-4'
          onExport={() => {
            // TODO: Implement option export functionality
          }}
        />

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

        {loading ? (
          <div className='text-center text-muted-foreground py-8'>
            <Trans>Loading options...</Trans>
          </div>
        ) : viewMode === 'grid' ? (
          <OptionGridView
            options={options}
            updateOptions={updateOptions}
            showHidden={showHiddenOptions}
          />
        ) : (
          <div className='mt-4'>
            <OptionListView
              options={options}
              updateOptions={updateOptions}
              showHidden={showHiddenOptions}
              actionHandlers={optionActionHandlers}
            />
          </div>
        )}

        {total > limit && (
          <div className='flex justify-center mt-6'>
            <div className='flex items-center gap-2'>
              <Button
                variant='outline'
                size='sm'
                disabled={page === 1}
                onClick={() => setParams({ page: page - 1 })}
              >
                <Trans>Previous</Trans>
              </Button>
              <span className='text-sm text-muted-foreground'>
                {t`Page ${page} of ${Math.ceil(total / limit)}`}
              </span>
              <Button
                variant='outline'
                size='sm'
                disabled={page >= Math.ceil(total / limit)}
                onClick={() => setParams({ page: page + 1 })}
              >
                <Trans>Next</Trans>
              </Button>
            </div>
          </div>
        )}
      </Container>

      {/* Dialogs for list view actions */}
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
          setIsExercising(false);
          setSelectedOption(null);
        }}
        onConfirm={() => updateOptions()}
        additionalData={
          isTransferring && response && selectedOption
            ? {
                title: t`Transfer Option`,
                content: (
                  <OptionConfirmation
                    options={[selectedOption]}
                    address={transferAddress}
                    type='transfer'
                  />
                ),
              }
            : isBurning && response && selectedOption
              ? {
                  title: t`Burn Option`,
                  content: (
                    <OptionConfirmation
                      options={[selectedOption]}
                      type='burn'
                    />
                  ),
                }
              : isExercising && response && selectedOption
                ? {
                    title: t`Exercise Option`,
                    content: (
                      <OptionConfirmation
                        options={[selectedOption]}
                        type='exercise'
                      />
                    ),
                  }
                : undefined
        }
      />
    </>
  );
}
