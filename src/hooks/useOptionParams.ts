import { ViewMode } from '@/components/ViewToggle';
import { useSearchParams } from 'react-router-dom';
import { useLocalStorage } from 'usehooks-ts';

const OPTION_SORT_STORAGE_KEY = 'sage-wallet-option-sort';
const OPTION_VIEW_MODE_STORAGE_KEY = 'sage-wallet-option-view-mode';
const HIDDEN_OPTIONS_STORAGE_KEY = 'sage-wallet-show-hidden-options';

export interface OptionParams {
  viewMode: ViewMode;
  sortMode: OptionSortMode;
  showHiddenOptions: boolean;
  search: string;
}

export enum OptionSortMode {
  Name = 'name',
  Balance = 'balance',
}

export function parseSortMode(view: string): OptionSortMode {
  switch (view) {
    case 'name':
      return OptionSortMode.Name;
    case 'balance':
      return OptionSortMode.Balance;
    default:
      return OptionSortMode.Name;
  }
}

export type SetOptionParams = (params: Partial<OptionParams>) => void;

export function useOptionParams(): [OptionParams, SetOptionParams] {
  const [params, setParams] = useSearchParams();

  const [storedTokenView, setStoredTokenView] = useLocalStorage<OptionSortMode>(
    OPTION_SORT_STORAGE_KEY,
    OptionSortMode.Name,
  );

  const [storedViewMode, setStoredViewMode] = useLocalStorage<ViewMode>(
    OPTION_VIEW_MODE_STORAGE_KEY,
    'grid',
  );

  const [storedShowHiddenOptions, setStoredShowHiddenOptions] =
    useLocalStorage<boolean>(HIDDEN_OPTIONS_STORAGE_KEY, false);

  const sortMode = parseSortMode(params.get('sortMode') ?? storedTokenView);
  const showHiddenOptions =
    (params.get('showHiddenOptions') ??
      storedShowHiddenOptions.toString()) === 'true';
  const search = params.get('search') ?? '';

  const viewMode = (params.get('viewMode') as ViewMode) ?? storedViewMode;

  const updateParams = ({
    sortMode,
    showHiddenOptions,
    search,
    viewMode,
  }: Partial<OptionParams>) => {
    setParams(
      (prev) => {
        const next = new URLSearchParams(prev);

        if (sortMode !== undefined) {
          next.set('sortMode', sortMode);
          setStoredTokenView(sortMode);
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

        return next;
      },
      { replace: true },
    );
  };

  return [
    {
      viewMode,
      sortMode,
      showHiddenOptions,
      search,
    },
    updateParams,
  ];
}
