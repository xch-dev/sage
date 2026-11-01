import { commands, OptionRecord, TransactionResponse } from '@/bindings';
import ConfirmationDialog from '@/components/ConfirmationDialog';
import { OptionConfirmation } from '@/components/confirmations/OptionConfirmation';
import { FeeOnlyDialog } from '@/components/FeeOnlyDialog';
import { TransferDialog } from '@/components/TransferDialog';
import { withHandleConfirmation } from '@/lib/handles';
import { useErrors } from '@/hooks/useErrors';
import { toMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { useCallback, useState } from 'react';

export interface OptionActionHandlers {
  onExercise: (option: OptionRecord) => void;
  onCancel: (option: OptionRecord) => void;
  onTransfer: (option: OptionRecord) => void;
  onBurn: (option: OptionRecord) => void;
  onToggleVisibility: (option: OptionRecord) => void;
}

type OptionAction = 'exercise' | 'cancel' | 'transfer' | 'burn';

export function useOptionActions(updateOptions: () => void) {
  const { addError } = useErrors();
  const walletState = useWalletState();

  // Dialog states
  const [selectedOption, setSelectedOption] = useState<OptionRecord | null>(
    null,
  );
  const [response, setResponse] = useState<TransactionResponse | null>(null);
  const [exerciseOpen, setExerciseOpen] = useState(false);
  const [cancelOpen, setCancelOpen] = useState(false);
  const [transferOpen, setTransferOpen] = useState(false);
  const [burnOpen, setBurnOpen] = useState(false);
  const [pendingAction, setPendingAction] = useState<OptionAction | null>(null);
  const [transferAddress, setTransferAddress] = useState('');

  // Action handler functions
  const submit = useCallback(
    (
      action: OptionAction,
      setOpen: (open: boolean) => void,
      request: () => Promise<TransactionResponse>,
    ) => {
      setPendingAction(action);
      request()
        .then(setResponse)
        .catch((err) => {
          setPendingAction(null);
          addError(err);
        })
        .finally(() => setOpen(false));
    },
    [addError],
  );

  const onExerciseSubmit = useCallback(
    (fee: string) => {
      if (!selectedOption) return;
      submit('exercise', setExerciseOpen, () =>
        commands.exerciseOptions({
          option_ids: [selectedOption.launcher_id],
          fee: toMojos(fee, walletState.sync.unit.precision),
        }),
      );
    },
    [selectedOption, walletState.sync.unit.precision, submit],
  );

  const onCancelSubmit = useCallback(
    (fee: string) => {
      if (!selectedOption) return;
      submit('cancel', setCancelOpen, () =>
        commands.cancelOptions({
          option_ids: [selectedOption.launcher_id],
          fee: toMojos(fee, walletState.sync.unit.precision),
        }),
      );
    },
    [selectedOption, walletState.sync.unit.precision, submit],
  );

  const onTransferSubmit = useCallback(
    (address: string, fee: string) => {
      if (!selectedOption) return;
      setTransferAddress(address);
      submit('transfer', setTransferOpen, () =>
        withHandleConfirmation((confirmed_handles) =>
          commands.transferOptions({
            option_ids: [selectedOption.launcher_id],
            address,
            fee: toMojos(fee, walletState.sync.unit.precision),
            confirmed_handles,
          }),
        ),
      );
    },
    [selectedOption, walletState.sync.unit.precision, submit],
  );

  const onBurnSubmit = useCallback(
    (fee: string) => {
      if (!selectedOption) return;
      submit('burn', setBurnOpen, () =>
        commands.transferOptions({
          option_ids: [selectedOption.launcher_id],
          address: walletState.sync.burn_address,
          fee: toMojos(fee, walletState.sync.unit.precision),
        }),
      );
    },
    [
      selectedOption,
      walletState.sync.unit.precision,
      walletState.sync.burn_address,
      submit,
    ],
  );

  // Action handlers object
  const actionHandlers: OptionActionHandlers = {
    onExercise: (option) => {
      setSelectedOption(option);
      setExerciseOpen(true);
    },
    onCancel: (option) => {
      setSelectedOption(option);
      setCancelOpen(true);
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

  const confirmationTitles: Record<OptionAction, string> = {
    exercise: t`Exercise Option`,
    cancel: t`Cancel Option`,
    transfer: t`Transfer Option`,
    burn: t`Burn Option`,
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

      <FeeOnlyDialog
        title={t`Cancel Option`}
        submitButtonLabel={t`Cancel Option`}
        open={cancelOpen}
        setOpen={setCancelOpen}
        onSubmit={onCancelSubmit}
      >
        <Trans>
          This will cancel the option contract and return the underlying asset
          to you. The strike price is not paid, but unless the underlying asset
          is the same asset, you need a coin of the strike asset and an XCH
          coin, of any amount.
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
          setPendingAction(null);
          setSelectedOption(null);
        }}
        onConfirm={() => updateOptions()}
        additionalData={
          pendingAction && response && selectedOption
            ? {
                title: confirmationTitles[pendingAction],
                content: (
                  <OptionConfirmation
                    options={[selectedOption]}
                    address={
                      pendingAction === 'transfer' ? transferAddress : undefined
                    }
                    type={pendingAction}
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
