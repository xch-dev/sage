import { commands, events, OptionRecord } from '@/bindings';
import Container from '@/components/Container';
import Header from '@/components/Header';
import { OptionGridView } from '@/components/OptionGridView';
import { OptionOptions } from '@/components/OptionOptions';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Switch } from '@/components/ui/switch';
import { CustomError } from '@/contexts/ErrorContext';
import { useErrors } from '@/hooks/useErrors';
import { t } from '@lingui/core/macro';
import { Plural, Trans } from '@lingui/react/macro';
import { FilePenLine } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';

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

  const hasHiddenOptions = options.findIndex((option) => !option.visible) > -1;

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
          className='mb-4'
          showHiddenOptions={showHidden}
          setShowHiddenOptions={setShowHidden}
        />

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

        <OptionGridView
          options={options}
          updateOptions={updateOptions}
          showHidden={showHidden}
        />
      </Container>
    </>
  );
}
