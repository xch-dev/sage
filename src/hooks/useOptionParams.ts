import { OptionSortMode } from '@/bindings';
import { ViewMode } from '@/components/ViewToggle';
import { useSearchParams } from 'react-router-dom';
import { useLocalStorage } from 'usehooks-ts';

const OPTION_SORT_STORAGE_KEY = 'sage-wallet-option-sort';
const OPTION_VIEW_MODE_STORAGE_KEY = 'sage-wallet-option-view-mode';
const HIDDEN_OPTIONS_STORAGE_KEY = 'sage-wallet-show-hidden-options';
const OPTION_ASCENDING_STORAGE_KEY = 'sage-wallet-option-ascending';

export interface OptionParams {
  viewMode: ViewMode;
  sortMode: OptionSortMode;
  ascending: boolean;
  showHiddenOptions: boolean;
  search: string;
  page: number;
  limit: number;
}

export function parseSortMode(mode: string): OptionSortMode {
  switch (mode) {
    case 'name':
      return 'name';
    case 'created_height':
      return 'created_height';
    case 'expiration_seconds':
      return 'expiration_seconds';
    default:
      return 'name';
  }
}

export type SetOptionParams = (params: Partial<OptionParams>) => void;

export function useOptionParams(): [OptionParams, SetOptionParams] {
  const [params, setParams] = useSearchParams();

  const [storedSortMode, setStoredSortMode] = useLocalStorage<OptionSortMode>(
    OPTION_SORT_STORAGE_KEY,
    'name',
  );

  const [storedViewMode, setStoredViewMode] = useLocalStorage<ViewMode>(
    OPTION_VIEW_MODE_STORAGE_KEY,
    'grid',
  );

  const [storedShowHiddenOptions, setStoredShowHiddenOptions] =
    useLocalStorage<boolean>(HIDDEN_OPTIONS_STORAGE_KEY, false);

  const [storedAscending, setStoredAscending] = useLocalStorage<boolean>(
    OPTION_ASCENDING_STORAGE_KEY,
    true,
  );

  const sortMode = parseSortMode(params.get('sortMode') ?? storedSortMode);
  const ascending =
    (params.get('ascending') ?? storedAscending.toString()) === 'true';
  const showHiddenOptions =
    (params.get('showHiddenOptions') ?? storedShowHiddenOptions.toString()) ===
    'true';
  const search = params.get('search') ?? '';
  const page = parseInt(params.get('page') ?? '1', 10);
  const limit = parseInt(params.get('limit') ?? '20', 10);

  const viewMode = (params.get('viewMode') as ViewMode) ?? storedViewMode;

  const updateParams = ({
    sortMode,
    ascending,
    showHiddenOptions,
    search,
    viewMode,
    page,
    limit,
  }: Partial<OptionParams>) => {
    setParams(
      (prev) => {
        const next = new URLSearchParams(prev);

        if (sortMode !== undefined) {
          next.set('sortMode', sortMode);
          setStoredSortMode(sortMode);
        }

        if (ascending !== undefined) {
          next.set('ascending', ascending.toString());
          setStoredAscending(ascending);
        }

        if (showHiddenOptions !== undefined) {
          next.set('showHiddenOptions', showHiddenOptions.toString());
          setStoredShowHiddenOptions(showHiddenOptions);
        }

        if (search !== undefined) {
          if (search) {
            next.set('search', search);
          } else {
            next.delete('search');
          }
        }

        if (viewMode !== undefined) {
          next.set('viewMode', viewMode);
          setStoredViewMode(viewMode);
        }

        if (page !== undefined) {
          next.set('page', page.toString());
        }

        if (limit !== undefined) {
          next.set('limit', limit.toString());
        }

        return next;
      },
      { replace: true },
    );
  };

  return [
    {
      viewMode,
      sortMode,
      ascending,
      showHiddenOptions,
      search,
      page,
      limit,
    },
    updateParams,
  ];
}
