import { commands, events, OptionFilter, OptionRecord } from '@/bindings';
import Container from '@/components/Container';
import Header from '@/components/Header';
import { OptionGridView } from '@/components/OptionGridView';
import { OptionListView } from '@/components/OptionListView';
import { OptionOptions } from '@/components/OptionOptions';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { ReadOnlyButton } from '@/components/ReadOnlyButton';
import { Button } from '@/components/ui/button';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { CustomError } from '@/contexts/ErrorContext';
import { useErrors } from '@/hooks/useErrors';
import { useOptionActions } from '@/hooks/useOptionActions';
import { useOptionParams } from '@/hooks/useOptionParams';
import { exportOptions } from '@/lib/exportOptions';
import { t } from '@lingui/core/macro';
import { Plural, Trans } from '@lingui/react/macro';
import { FilePenLine, FilterIcon } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';

export function OptionList() {
  const navigate = useNavigate();
  const { addError } = useErrors();
  const [params, setParams] = useOptionParams();
  const {
    viewMode,
    sortMode,
    ascending,
    showHiddenOptions,
    filter,
    search,
    page,
    limit,
  } = params;
  const [options, setOptions] = useState<OptionRecord[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(false);

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
        filter,
      });

      setOptions(data.options);
      setTotal(data.total);
    } catch (error) {
      addError(error as CustomError);
    } finally {
      setLoading(false);
    }
  }, [
    addError,
    page,
    limit,
    sortMode,
    ascending,
    search,
    showHiddenOptions,
    filter,
  ]);

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

  // Shared action handlers and dialogs
  const { actionHandlers, dialogs } = useOptionActions(updateOptions);

  const pageCount = Math.ceil(total / limit);

  return (
    <>
      <Header title={t`Option Contracts`}>
        <div className='flex items-center gap-2'>
          <ReceiveAddress />
        </div>
      </Header>
      <Container>
        <div className='flex flex-wrap items-center justify-between gap-4 mb-4'>
          <ReadOnlyButton
            aria-label={t`Mint new option`}
            onClick={() => navigate('/options/mint')}
          >
            <FilePenLine className='h-4 w-4 mr-2' />
            <Trans>Mint Option</Trans>
          </ReadOnlyButton>

          <div className='flex items-center gap-2'>
            <FilterIcon
              className='h-4 w-4 text-muted-foreground'
              aria-hidden='true'
            />
            <Select
              value={filter}
              onValueChange={(value) =>
                setParams({ filter: value as OptionFilter, page: 1 })
              }
            >
              <SelectTrigger
                className='w-[180px]'
                aria-label={t`Filter options`}
              >
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value='owned'>
                  <Trans>Owned</Trans>
                </SelectItem>
                <SelectItem value='created'>
                  <Trans>Minted by me</Trans>
                </SelectItem>
                <SelectItem value='all'>
                  <Trans>All Options</Trans>
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

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
          onExport={() => exportOptions(options)}
        />

        {options.length === 0 && filter !== 'owned' && !loading && (
          <div className='text-center text-muted-foreground py-8'>
            <Trans>No options found.</Trans>
          </div>
        )}

        {options.length === 0 && filter === 'owned' && (
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
              actionHandlers={actionHandlers}
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
                {t`Page ${page} of ${pageCount}`}
              </span>
              <Button
                variant='outline'
                size='sm'
                disabled={page >= pageCount}
                onClick={() => setParams({ page: page + 1 })}
              >
                <Trans>Next</Trans>
              </Button>
            </div>
          </div>
        )}
      </Container>

      {dialogs}
    </>
  );
}
