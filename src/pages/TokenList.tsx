import Container from '@/components/Container';
import Header from '@/components/Header';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { TokenGridView } from '@/components/TokenGridView';
import { TokenListView } from '@/components/TokenListView';
import { TokenOptions } from '@/components/TokenOptions';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { useErrors } from '@/hooks/useErrors';
import { usePrices } from '@/hooks/usePrices';
import { TokenSortMode, useTokenParams } from '@/hooks/useTokenParams';
import { exportTokens } from '@/lib/exportTokens';
import { isValidAssetId, toDecimal } from '@/lib/utils';
import { TokenRecordWithPrices } from '@/types/TokenViewProps';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { Coins, InfoIcon } from 'lucide-react';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { toast } from 'react-toastify';
import { TokenRecord, commands, events } from '../bindings';
import { useWalletState } from '../state';
import { useTokenState } from '@/hooks/useTokenState';

export function TokenList() {
  const navigate = useNavigate();
  const walletState = useWalletState();
  const { getBalanceInUsd, getPriceInUsd } = usePrices();
  const { addError } = useErrors();
  const [params, setParams] = useTokenParams();
  const { viewMode, sortMode, showZeroBalanceTokens, showHiddenCats } = params;
  const [cats, setCats] = useState<TokenRecord[]>([]);

  const { asset: xchAsset } = useTokenState('xch');
  console.log(xchAsset);
  const xchRecord = useMemo(() => {
    if (!xchAsset) {
      return null;
    }

    return {
      ...xchAsset,
      balance: xchAsset.balance.toString(),
      balanceInUsd: Number(
        getBalanceInUsd(
          'xch',
          toDecimal(xchAsset.balance, walletState.sync.unit.decimals),
        ),
      ),
      priceInUsd: getPriceInUsd('xch'),
      decimals: walletState.sync.unit.decimals,
      isXch: true,
    };
  }, [
    xchAsset,
    getBalanceInUsd,
    getPriceInUsd,
    walletState.sync.unit.decimals,
  ]);

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
          decimals: 3,
          isXch: false,
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
    onEdit: (asset: TokenRecordWithPrices) => {
      navigate(`/wallet/token/${asset.asset_id}`);
    },
    onRefreshInfo: (assetId: string) => {
      if (assetId === 'xch') return;
      commands
        .resyncCat({ asset_id: assetId })
        .then(() => {
          updateCats();
          toast.success(t`Refreshing token info...`);
        })
        .catch(addError);
    },
    onToggleVisibility: (asset: TokenRecordWithPrices) => {
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
          onExport={() => xchRecord && exportTokens([xchRecord, ...filteredCats])}
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

        {xchRecord && (
          viewMode === 'grid' ? (
            <TokenGridView
              cats={filteredCats}
              xchRecord={xchRecord}
              actionHandlers={tokenActionHandlers}
            />
          ) : (
            <div className='mt-4'>
              <TokenListView
                cats={filteredCats}
                xchRecord={xchRecord}
                actionHandlers={tokenActionHandlers}
              />
            </div>
          )
        )}
      </Container>
    </>
  );
}
