import { commands, events, OptionRecord } from '@/bindings';
import Container from '@/components/Container';
import Header from '@/components/Header';
import { OptionGridView } from '@/components/OptionGridView';
import { OptionOptions } from '@/components/OptionOptions';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { CustomError } from '@/contexts/ErrorContext';
import { useErrors } from '@/hooks/useErrors';
import { useOptionParams } from '@/hooks/useOptionParams';
import { t } from '@lingui/core/macro';
import { Plural, Trans } from '@lingui/react/macro';
import { FilePenLine } from 'lucide-react';
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
            {/* TODO: Implement OptionListView */}
            <div className='text-center text-muted-foreground py-8'>
              <Trans>List view coming soon...</Trans>
            </div>
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
    </>
  );
}
