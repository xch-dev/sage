import { commands, events, OptionRecord } from '@/bindings';
import Container from '@/components/Container';
import Header from '@/components/Header';
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

  const visibleOptions = showHidden
    ? options
    : options.filter((option) => option.visible);
  const hasHiddenOptions = options.findIndex((option) => !option.visible) > -1;

  console.log(visibleOptions);

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
            <Option key={option.launcher_id} option={option} />
          ))}
        </div>
      </Container>
    </>
  );
}

interface OptionProps {
  option: OptionRecord;
}

function Option({ option }: OptionProps) {
  console.log(option);

  return <>Hi</>;
}
