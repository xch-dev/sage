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
import { Input } from '@/components/ui/input';
import { Switch } from '@/components/ui/switch';
import { useErrors } from '@/hooks/useErrors';
import { usePrices } from '@/hooks/usePrices';
import { useTokenParams } from '@/hooks/useTokenParams';
import { toDecimal } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import {
  ArrowDown10,
  ArrowDownAz,
  CircleDollarSign,
  CircleSlash,
  Coins,
  InfoIcon,
  SearchIcon,
  XIcon,
} from 'lucide-react';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { CatRecord, commands, events } from '../bindings';
import { useWalletState } from '../state';
import { NumberFormat } from '@/components/NumberFormat';
import { fromMojos } from '@/lib/utils';
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip';

enum TokenView {
  Name = 'name',
  Balance = 'balance',
}

export function TokenList() {
  const navigate = useNavigate();
  const walletState = useWalletState();
  const { getBalanceInUsd, getPriceInUsd } = usePrices();
  const { addError } = useErrors();
  const [params, setParams] = useTokenParams();
  const { view, showHidden, showZeroBalance } = params;
  const [cats, setCats] = useState<CatRecord[]>([]);

  const catsWithBalanceInUsd = useMemo(
    () =>
      cats.map((cat) => {
        const balance = Number(toDecimal(cat.balance, 3));
        const usdValue = parseFloat(
          getBalanceInUsd(cat.asset_id, balance.toString()),
        );

        return {
          ...cat,
          balanceInUsd: usdValue,
          sortValue: usdValue,
        };
      }),
    [cats, getBalanceInUsd],
  );

  const sortedCats = catsWithBalanceInUsd.sort((a, b) => {
    if (a.visible && !b.visible) return -1;
    if (!a.visible && b.visible) return 1;

    if (view === TokenView.Balance) {
      if (a.balanceInUsd === 0 && b.balanceInUsd === 0) {
        return (
          Number(toDecimal(b.balance, 3)) - Number(toDecimal(a.balance, 3))
        );
      }
      return b.sortValue - a.sortValue;
    }

    const aName = a.name || 'Unknown CAT';
    const bName = b.name || 'Unknown CAT';

    if (aName === 'Unknown CAT' && bName !== 'Unknown CAT') return 1;
    if (bName === 'Unknown CAT' && aName !== 'Unknown CAT') return -1;

    return aName.localeCompare(bName);
  });

  const filteredCats = sortedCats.filter((cat) => {
    if (!showHidden && !cat.visible) {
      return false;
    }

    if (!showZeroBalance && Number(toDecimal(cat.balance, 3)) === 0) {
      return false;
    }

    if (params.search) {
      const searchTerm = params.search.toLowerCase();
      const name = (cat.name || 'Unknown CAT').toLowerCase();
      const ticker = (cat.ticker || '').toLowerCase();
      return name.includes(searchTerm) || ticker.includes(searchTerm);
    }

    return true;
  });

  const hasHiddenAssets = !!sortedCats.find((cat) => !cat.visible);

  const updateCats = useCallback(
    () =>
      commands
        .getCats({})
        .then((data) => setCats(data.cats))
        .catch(addError),
    [addError],
  );

  useEffect(() => {
    updateCats();

    const unlisten = events.syncEvent.listen((event) => {
      const type = event.payload.type;

      if (
        type === 'coin_state' ||
        type === 'puzzle_batch_synced' ||
        type === 'cat_info'
      ) {
        updateCats();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [updateCats]);

  return (
    <>
      <Header title={<Trans>Assets</Trans>}>
        <div className='flex items-center gap-2'>
          <ReceiveAddress />
        </div>
      </Header>
      <Container>
        <Button onClick={() => navigate('/wallet/issue-token')}>
          <Coins className='h-4 w-4 mr-2' /> <Trans>Issue Token</Trans>
        </Button>

        <div className='flex items-center justify-between gap-2 mt-4'>
          <div className='relative flex-1'>
            <div className='relative'>
              <SearchIcon className='absolute left-2 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground' />
              <Input
                value={params.search}
                aria-label={t`Search for a token`}
                title={t`Search for a token`}
                placeholder={t`Search tokens...`}
                onChange={(e) => setParams({ search: e.target.value })}
                className='w-full pl-8 pr-8'
              />
            </div>
            {params.search && (
              <Button
                variant='ghost'
                size='icon'
                title={t`Clear search`}
                aria-label={t`Clear search`}
                className='absolute right-0 top-0 h-full px-2 hover:bg-transparent'
                onClick={() => setParams({ search: '' })}
              >
                <XIcon className='h-4 w-4' />
              </Button>
            )}
          </div>
          <TokenSortDropdown
            view={view}
            setView={(view) => setParams({ view })}
          />
          <Button
            variant='outline'
            size='icon'
            onClick={() => setParams({ showZeroBalance: !showZeroBalance })}
            className={!showZeroBalance ? 'text-muted-foreground' : ''}
            aria-label={
              showZeroBalance ? t`Hide zero balances` : t`Show zero balances`
            }
            title={
              showZeroBalance ? t`Hide zero balances` : t`Show zero balances`
            }
          >
            {showZeroBalance ? (
              <CircleDollarSign className='h-4 w-4' />
            ) : (
              <CircleSlash className='h-4 w-4' />
            )}
          </Button>
        </div>

        {walletState.sync.synced_coins < walletState.sync.total_coins && (
          <Alert className='mt-4 mb-4'>
            <InfoIcon className='h-4 w-4' />
            <AlertTitle>
              <Trans>Syncing in progress...</Trans>
            </AlertTitle>
            <AlertDescription>
              <Trans>
                The wallet is still syncing. Balances may not be accurate until
                it completes.
              </Trans>
            </AlertDescription>
          </Alert>
        )}

        <div className='flex items-center gap-4 my-4'>
          {hasHiddenAssets && (
            <div className='flex items-center gap-2'>
              <label htmlFor='viewHidden'>
                <Trans>View hidden</Trans>
              </label>
              <Switch
                id='viewHidden'
                checked={showHidden}
                onCheckedChange={(value) => setParams({ showHidden: value })}
              />
            </div>
          )}
        </div>

        <div className='mt-4 grid gap-2 md:gap-4 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4'>
          <Link to={`/wallet/token/xch`}>
            <Card className='transition-colors hover:bg-neutral-50 dark:hover:bg-neutral-900'>
              <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2'>
                <CardTitle className='text-md font-medium'>Chia</CardTitle>
                <img
                  alt={t`XCH logo`}
                  className='h-6 w-6'
                  src='https://icons.dexie.space/xch.webp'
                />
              </CardHeader>
              <CardContent>
                <div className='text-2xl font-medium truncate'>
                  <NumberFormat
                    value={fromMojos(
                      walletState.sync.balance,
                      walletState.sync.unit.decimals,
                    )}
                    minimumFractionDigits={0}
                    maximumFractionDigits={walletState.sync.unit.decimals}
                  />
                </div>
                <div className='text-sm text-neutral-500'>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <div>
                        ~
                        <NumberFormat
                          value={getBalanceInUsd(
                            'xch',
                            toDecimal(
                              walletState.sync.balance,
                              walletState.sync.unit.decimals,
                            ),
                          )}
                          style='currency'
                          currency='USD'
                          minimumFractionDigits={2}
                          maximumFractionDigits={2}
                        />
                      </div>
                    </TooltipTrigger>
                    <TooltipContent>
                      <span>1 XCH = ${getPriceInUsd('xch')}</span>
                    </TooltipContent>
                  </Tooltip>
                </div>
              </CardContent>
            </Card>
          </Link>
          {filteredCats.map((cat) => (
            <Link key={cat.asset_id} to={`/wallet/token/${cat.asset_id}`}>
              <Card
                className={`transition-colors hover:bg-neutral-50 dark:hover:bg-neutral-900 ${!cat.visible ? 'opacity-50 grayscale' : ''}`}
              >
                <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2 space-x-2'>
                  <CardTitle className='text-md font-medium truncate'>
                    {cat.name || t`Unknown CAT`}
                  </CardTitle>
                  {cat.icon_url && (
                    <img
                      alt={`${cat.asset_id} logo`}
                      className='h-6 w-6'
                      src={cat.icon_url}
                    />
                  )}
                </CardHeader>
                <CardContent>
                  <div className='text-2xl font-medium truncate'>
                    <NumberFormat
                      value={fromMojos(cat.balance, 3)}
                      minimumFractionDigits={0}
                      maximumFractionDigits={3}
                    />{' '}
                    {cat.ticker ?? ''}
                  </div>
                  <div className='text-sm text-neutral-500'>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <div>
                          ~
                          <NumberFormat
                            value={cat.balanceInUsd}
                            style='currency'
                            currency='USD'
                            minimumFractionDigits={2}
                            maximumFractionDigits={2}
                          />
                        </div>
                      </TooltipTrigger>
                      <TooltipContent>
                        <span>
                          1 {cat.ticker ?? 'CAT'}{' '}
                          {(() => {
                            const price = getPriceInUsd(cat.asset_id);
                            return price < 0.01 // prices less than a penny
                              ? ' < 0.01¢'
                              : price < 1 // prices less than a dollar
                                ? ` = ${(price * 100).toFixed(2)}¢`
                                : ` = $${new Number(price).toFixed(2)}`;
                          })()}
                        </span>
                      </TooltipContent>
                    </Tooltip>
                  </div>
                </CardContent>
              </Card>
            </Link>
          ))}
        </div>
      </Container>
    </>
  );
}

function TokenSortDropdown({
  view,
  setView,
}: {
  view: TokenView;
  setView: (view: TokenView) => void;
}) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant='outline' size='icon' title={t`Sort options`}>
          {view === TokenView.Balance ? (
            <ArrowDown10 className='h-4 w-4' />
          ) : (
            <ArrowDownAz className='h-4 w-4' />
          )}
        </Button>
      </DropdownMenuTrigger>

      <DropdownMenuContent align='end'>
        <DropdownMenuGroup>
          <DropdownMenuItem
            className='cursor-pointer'
            onClick={(e) => {
              e.stopPropagation();
              setView(TokenView.Name);
            }}
          >
            <ArrowDownAz className='mr-2 h-4 w-4' />
            <span>
              <Trans>Sort Alphabetically</Trans>
            </span>
          </DropdownMenuItem>

          <DropdownMenuItem
            className='cursor-pointer'
            onClick={(e) => {
              e.stopPropagation();
              setView(TokenView.Balance);
            }}
          >
            <ArrowDown10 className='mr-2 h-4 w-4' />
            <span>
              <Trans>Sort by Balance</Trans>
            </span>
          </DropdownMenuItem>
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
