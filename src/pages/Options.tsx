import Container from '@/components/Container';
import Header from '@/components/Header';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Switch } from '@/components/ui/switch';
import { useErrors } from '@/hooks/useErrors';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import {
  Copy,
  EyeIcon,
  EyeOff,
  MoreVerticalIcon,
  UserIcon,
  UserPlusIcon,
  UserRoundPlus,
} from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { toast } from 'react-toastify';
import { commands, OptionRecord } from '../bindings';

export function Options() {
  const { addError } = useErrors();

  const navigate = useNavigate();

  const [showHidden, setShowHidden] = useState(false);

  const [options, setOptions] = useState<OptionRecord[]>([]);

  const updateOptions = useCallback(() => {
    commands
      .getOptions({ include_hidden: showHidden })
      .then((response) => setOptions(response.options))
      .catch(addError);
  }, [showHidden, addError]);

  useEffect(() => {
    updateOptions();
  }, [showHidden, updateOptions]);

  const visibleOptions = showHidden
    ? options
    : options.filter((option) => option.visible);
  const hasHiddenOptions = options.findIndex((option) => !option.visible) > -1;

  return (
    <>
      <Header title={t`Options`}>
        <ReceiveAddress />
      </Header>
      <Container>
        <Button onClick={() => navigate('/options/create')}>
          <UserPlusIcon className='h-4 w-4 mr-2' />
          <Trans>Create Option</Trans>
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
            <UserRoundPlus className='h-4 w-4' />
            <AlertTitle>
              <Trans>Create an option?</Trans>
            </AlertTitle>
            <AlertDescription>
              <Trans>
                You do not currently have any options. Would you like to create
                one?
              </Trans>
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

  const toggleVisibility = () => {
    commands
      .updateOption({
        option_id: option.launcher_id,
        visible: !option.visible,
      })
      .then(updateOptions)
      .catch(addError);
  };

  return (
    <>
      <Card
        key={option.launcher_id}
        className={`${!option.visible ? 'opacity-50 grayscale' : option.create_transaction_id !== null ? 'pulsate-opacity' : ''}`}
      >
        <CardHeader className='-mt-2 flex flex-row items-center justify-between space-y-0 pb-2 pr-2 space-x-2'>
          <CardTitle className='text-md font-medium truncate flex items-center'>
            <UserIcon className='mr-2 h-4 w-4' />
            <Trans>Option</Trans>
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
                    toggleVisibility();
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
          <div className='text-sm font-medium truncate'>
            {option.launcher_id}
          </div>
        </CardContent>
      </Card>
    </>
  );
}
