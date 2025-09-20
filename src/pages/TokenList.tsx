import Container from '@/components/Container';
import Header from '@/components/Header';
import { NumberFormat } from '@/components/NumberFormat';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { TokenGridView } from '@/components/TokenGridView';
import { TokenListView } from '@/components/TokenListView';
import { TokenOptions } from '@/components/TokenOptions';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { useErrors } from '@/hooks/useErrors';
import { usePrices } from '@/hooks/usePrices';
import { TokenSortMode, useTokenParams } from '@/hooks/useTokenParams';
import { exportTokens } from '@/lib/exportTokens';
import { isValidAssetId, toDecimal } from '@/lib/utils';
import { PricedTokenRecord } from '@/types/TokenViewProps';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { Coins, InfoIcon, ShieldAlert } from 'lucide-react';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { toast } from 'react-toastify';
import { commands, events, TokenRecord } from '../bindings';
import { useWalletState } from '../state';

export function TokenList() {
  const navigate = useNavigate();
  const walletState = useWalletState();
  const { getBalanceInUsd, getPriceInUsd } = usePrices();
  const { addError } = useErrors();
  const [params, setParams] = useTokenParams();
  const { viewMode, sortMode, showZeroBalanceTokens, showHiddenCats } = params;
  const [tokens, setTokens] = useState<TokenRecord[]>([]);

  const pricedTokens = useMemo(
    () =>
      tokens.map((token): PricedTokenRecord & { sortValue: number } => {
        const balance = Number(toDecimal(token.balance, token.precision));
        const usdValue = parseFloat(
          getBalanceInUsd(token.asset_id, balance.toString()),
        );

        return {
          ...token,
          balanceInUsd: usdValue,
          sortValue: usdValue,
          priceInUsd: getPriceInUsd(token.asset_id),
        };
      }),
    [tokens, getBalanceInUsd, getPriceInUsd],
  )
    .sort((a, b) => {
      if (!a.asset_id && b.asset_id) return -1;
      if (a.asset_id && !b.asset_id) return 1;

      if (a.visible && !b.visible) return -1;
      if (!a.visible && b.visible) return 1;

      if (sortMode === TokenSortMode.Balance) {
        if (a.balanceInUsd !== b.balanceInUsd) {
          return b.balanceInUsd - a.balanceInUsd;
        }
        return (
          Number(toDecimal(b.balance, 3)) - Number(toDecimal(a.balance, 3))
        );
      }

      const aName = a.name || 'Unknown CAT';
      const bName = b.name || 'Unknown CAT';

      if (aName === 'Unknown CAT' && bName !== 'Unknown CAT') return 1;
      if (bName === 'Unknown CAT' && aName !== 'Unknown CAT') return -1;

      return aName.localeCompare(bName);
    })
    .filter((token) => {
      if (!token.asset_id) return true;

      if (!showHiddenCats && !token.visible) {
        return false;
      }

      if (!showZeroBalanceTokens && Number(toDecimal(token.balance, 3)) === 0) {
        return false;
      }

      if (params.search) {
        if (isValidAssetId(params.search)) {
          return token.asset_id?.toLowerCase() === params.search.toLowerCase();
        }

        const searchTerm = params.search.toLowerCase();
        const name = (token.name || 'Unknown CAT').toLowerCase();
        const ticker = (token.ticker || '').toLowerCase();
        return name.includes(searchTerm) || ticker.includes(searchTerm);
      }

      return true;
    });

  const totalUsdBalance = useMemo(() => {
    // Use the filtered pricedTokens array which already respects all filters including search
    return pricedTokens.reduce((total, token) => {
      return total + (token.balanceInUsd || 0);
    }, 0);
  }, [pricedTokens]);

  const updateCats = useCallback(
    () =>
      Promise.all([commands.getToken({ asset_id: null }), commands.getCats({})])
        .then(([xch, data]) =>
          setTokens([...(xch.token ? [xch.token] : []), ...data.cats]),
        )
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
      navigate(`/wallet/token/${asset.asset_id ?? 'xch'}`);
    },
    onRefreshInfo: (assetId: string | null) => {
      if (!assetId) return;
      commands
        .resyncCat({ asset_id: assetId })
        .then(() => {
          updateCats();
          toast.success(t`Refreshing token info...`);
        })
        .catch(addError);
    },
    onToggleVisibility: (asset: TokenRecord) => {
      if (!asset.asset_id) return;
      const updatedToken = tokens.find(
        (token) => token.asset_id === asset.asset_id,
      );
      if (!updatedToken) return;

      updatedToken.visible = !updatedToken.visible;
      commands
        .updateCat({ record: updatedToken })
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
          onExport={() => exportTokens(pricedTokens)}
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
            tokens={pricedTokens}
            actionHandlers={tokenActionHandlers}
          />
        ) : (
          <div className='mt-4'>
            <TokenListView
              tokens={pricedTokens}
              actionHandlers={tokenActionHandlers}
            />
          </div>
        )}

        {pricedTokens.length > 0 && (
          <div className='mt-6 pt-4 border-t border-border'>
            <div className='flex justify-between items-center font-semibold text-lg'>
              <div className='flex items-center gap-2'>
                <span>
                  <Trans>Total Estimated Balance</Trans>
                </span>
                <TooltipProvider>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <ShieldAlert className='h-4 w-4 text-muted-foreground cursor-help' />
                    </TooltipTrigger>
                    <TooltipContent>
                      <p>
                        <Trans>
                          This is an estimate only and does not account for
                          liquidity
                        </Trans>
                      </p>
                    </TooltipContent>
                  </Tooltip>
                </TooltipProvider>
              </div>
              <span className='text-primary'>
                <NumberFormat
                  value={totalUsdBalance}
                  style='currency'
                  currency='USD'
                  maximumFractionDigits={2}
                />
              </span>
            </div>
          </div>
        )}
      </Container>
    </>
  );
}
