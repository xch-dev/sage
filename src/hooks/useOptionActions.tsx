import { commands, OptionRecord, TransactionResponse } from '@/bindings';
import ConfirmationDialog from '@/components/ConfirmationDialog';
import { OptionConfirmation } from '@/components/confirmations/OptionConfirmation';
import { FeeOnlyDialog } from '@/components/FeeOnlyDialog';
import { TransferDialog } from '@/components/TransferDialog';
import { useErrors } from '@/hooks/useErrors';
import { toMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { useCallback, useState } from 'react';

export interface OptionActionHandlers {
  onExercise: (option: OptionRecord) => void;
  onTransfer: (option: OptionRecord) => void;
  onBurn: (option: OptionRecord) => void;
  onToggleVisibility: (option: OptionRecord) => void;
}

export function useOptionActions(updateOptions: () => void) {
  const { addError } = useErrors();
  const walletState = useWalletState();

  // Dialog states
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

  // Action handler functions
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

  // Action handlers object
  const actionHandlers: OptionActionHandlers = {
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

  // Dialog components
  const dialogs = (
    <>
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

  return {
    actionHandlers,
    dialogs,
  };
}
