import Container from '@/components/Container';
import Header from '@/components/Header';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { useErrors } from '@/hooks/useErrors';
import { usePrices } from '@/hooks/usePrices';
import { useTokenParams } from '@/hooks/useTokenParams';
import { toDecimal, isValidAssetId } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { Coins, InfoIcon } from 'lucide-react';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { CatRecord, commands, events } from '../bindings';
import { useWalletState } from '../state';
import { TokenListView } from '@/components/TokenListView';
import { TokenGridView } from '@/components/TokenGridView';
import { TokenOptions } from '@/components/TokenOptions';
import { TokenSortMode } from '@/hooks/useTokenParams';
import { TokenRecord } from '@/types/TokenViewProps';
import { toast } from 'react-toastify';

export function TokenList() {
  const navigate = useNavigate();
  const walletState = useWalletState();
  const { getBalanceInUsd, getPriceInUsd } = usePrices();
  const { addError } = useErrors();
  const [params, setParams] = useTokenParams();
  const { viewMode, sortMode, showZeroBalanceTokens, showHiddenCats } = params;
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
          priceInUsd: getPriceInUsd(cat.asset_id),
        };
      }),
    [cats, getBalanceInUsd, getPriceInUsd],
  );

  const sortedCats = catsWithBalanceInUsd.sort((a, b) => {
    if (a.visible && !b.visible) return -1;
    if (!a.visible && b.visible) return 1;

    if (sortMode === TokenSortMode.Balance) {
      if (a.balanceInUsd !== b.balanceInUsd) {
        return b.balanceInUsd - a.balanceInUsd;
      }
      return Number(toDecimal(b.balance, 3)) - Number(toDecimal(a.balance, 3));
    }

    const aName = a.name || 'Unknown CAT';
    const bName = b.name || 'Unknown CAT';

    if (aName === 'Unknown CAT' && bName !== 'Unknown CAT') return 1;
    if (bName === 'Unknown CAT' && aName !== 'Unknown CAT') return -1;

    return aName.localeCompare(bName);
  });

  const filteredCats = sortedCats.filter((cat) => {
    if (!showHiddenCats && !cat.visible) {
      return false;
    }

    if (!showZeroBalanceTokens && Number(toDecimal(cat.balance, 3)) === 0) {
      return false;
    }

    if (params.search) {
      if (isValidAssetId(params.search)) {
        return cat.asset_id.toLowerCase() === params.search.toLowerCase();
      }

      const searchTerm = params.search.toLowerCase();
      const name = (cat.name || 'Unknown CAT').toLowerCase();
      const ticker = (cat.ticker || '').toLowerCase();
      return name.includes(searchTerm) || ticker.includes(searchTerm);
    }

    return true;
  });

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

  const tokenActionHandlers = {
    onEdit: (asset: TokenRecord) => {
      navigate(`/wallet/token/${asset.asset_id}`);
    },
    onRefreshInfo: (assetId: string) => {
      if (assetId === 'xch') return;
      commands
        .removeCat({ asset_id: assetId })
        .then(() => {
          updateCats();
          toast.success(t`Refreshing token info...`);
        })
        .catch(addError);
    },
    onToggleVisibility: (asset: TokenRecord) => {
      if (asset.asset_id === 'xch') return;
      const updatedCat = cats.find((cat) => cat.asset_id === asset.asset_id);
      if (!updatedCat) return;

      updatedCat.visible = !updatedCat.visible;
      commands
        .updateCat({ record: updatedCat })
        .then(() => updateCats())
        .catch(addError);
    },
  };

  return (
    <>
      <Header title={<Trans>Assets</Trans>}>
        <div className='flex items-center gap-2'>
          <ReceiveAddress />
        </div>
      </Header>
      <Container>
        <Button
          onClick={() => navigate('/wallet/issue-token')}
          aria-label={t`Issue new token`}
          className='mb-4'
        >
          <Coins className='h-4 w-4 mr-2' aria-hidden='true' />
          <Trans>Issue Token</Trans>
        </Button>

        <TokenOptions
          query={params.search}
          setQuery={(value) => setParams({ search: value })}
          viewMode={viewMode}
          setViewMode={(value) => setParams({ viewMode: value })}
          sortMode={sortMode}
          setSortMode={(value) => setParams({ sortMode: value })}
          showZeroBalanceTokens={showZeroBalanceTokens}
          setShowZeroBalanceTokens={(value) =>
            setParams({ showZeroBalanceTokens: value })
          }
          showHiddenCats={showHiddenCats}
          setShowHiddenCats={(value) => setParams({ showHiddenCats: value })}
          handleSearch={(value) => {
            setParams({ search: value });
          }}
          className='mb-4'
        />

        {walletState.sync.synced_coins < walletState.sync.total_coins && (
          <Alert className='mt-4 mb-4' role='status'>
            <InfoIcon className='h-4 w-4' aria-hidden='true' />
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

        {viewMode === 'grid' ? (
          <TokenGridView
            cats={filteredCats}
            xchBalance={walletState.sync.balance.toString()}
            xchDecimals={walletState.sync.unit.decimals}
            xchPrice={getPriceInUsd('xch')}
            xchBalanceUsd={Number(
              getBalanceInUsd(
                'xch',
                toDecimal(
                  walletState.sync.balance,
                  walletState.sync.unit.decimals,
                ),
              ),
            )}
            actionHandlers={tokenActionHandlers}
          />
        ) : (
          <div className='mt-4'>
            <TokenListView
              cats={filteredCats}
              xchBalance={walletState.sync.balance.toString()}
              xchDecimals={walletState.sync.unit.decimals}
              xchPrice={getPriceInUsd('xch')}
              xchBalanceUsd={Number(
                getBalanceInUsd(
                  'xch',
                  toDecimal(
                    walletState.sync.balance,
                    walletState.sync.unit.decimals,
                  ),
                ),
              )}
              actionHandlers={tokenActionHandlers}
            />
          </div>
        )}
      </Container>
    </>
  );
}
