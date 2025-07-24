import ConfirmationDialog from '@/components/ConfirmationDialog';
import Container from '@/components/Container';
import { CopyButton } from '@/components/CopyButton';
import Header from '@/components/Header';
import { OwnedCoinsCard } from '@/components/OwnedCoinsCard';
import { TokenCard } from '@/components/TokenCard';
import { TokenConfirmation } from '@/components/confirmations/TokenConfirmation';
import { useErrors } from '@/hooks/useErrors';
import { usePrices } from '@/hooks/usePrices';
import { getAssetDisplayName, toDecimal } from '@/lib/utils';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { RowSelectionState } from '@tanstack/react-table';
import { useEffect, useMemo, useRef, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { toast } from 'react-toastify';
import {
  CoinRecord,
  CoinSortMode,
  commands,
  events,
  TokenRecord,
  TransactionResponse,
} from '../bindings';

interface EnhancedTransactionResponse extends TransactionResponse {
  additionalData?: {
    title: string;
    content: {
      type: 'split' | 'combine';
      coins: CoinRecord[];
      outputCount?: number;
      ticker: string;
      precision: number;
    };
  };
}

export default function Token() {
  let { asset_id: assetId = null } = useParams();
  if (assetId === 'xch') assetId = null;

  const walletState = useWalletState();

  const { addError } = useErrors();
  const { getBalanceInUsd } = usePrices();

  const [asset, setAsset] = useState<TokenRecord | null>(null);
  const [response, setResponse] = useState<EnhancedTransactionResponse | null>(
    null,
  );
  const [coins, setCoins] = useState<CoinRecord[]>([]);
  const [selectedCoins, setSelectedCoins] = useState<RowSelectionState>({});
  const [currentPage, setCurrentPage] = useState<number>(0);
  const [totalCoins, setTotalCoins] = useState<number>(0);
  const [sortMode, setSortMode] = useState<CoinSortMode>('created_height');
  const [sortDirection, setSortDirection] = useState<boolean>(false); // false = descending, true = ascending
  const [includeSpentCoins, setIncludeSpentCoins] = useState<boolean>(false);
  const pageSize = 10;

  const navigate = useNavigate();

  const updateToken = useMemo(
    () => () => {
      commands
        .getToken({ asset_id: assetId ?? null })
        .then((res) => setAsset(res.token))
        .catch(addError);
    },
    [assetId, addError],
  );

  useEffect(() => {
    updateToken();

    const unlisten = events.syncEvent.listen((event) => {
      const type = event.payload.type;

      if (
        type === 'coin_state' ||
        type === 'puzzle_batch_synced' ||
        type === 'cat_info'
      ) {
        updateToken();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [assetId, updateToken, walletState.sync]);

  const balanceInUsd = useMemo(() => {
    if (!asset) return '0';
    return getBalanceInUsd(
      asset.asset_id,
      toDecimal(asset.balance, asset.precision),
    );
  }, [asset, getBalanceInUsd]);

  const updateCoins = useMemo(
    () =>
      (page: number = currentPageRef.current) => {
        const offset = page * pageSize;

        commands
          .getCoins({
            asset_id: assetId,
            offset,
            limit: pageSize,
            sort_mode: sortMode,
            ascending: sortDirection,
            filter_mode: includeSpentCoins ? 'spent' : 'owned',
          })
          .then((res) => {
            setCoins(res.coins);
            setTotalCoins(res.total);
          })
          .catch(addError);
      },
    [assetId, addError, pageSize, sortMode, sortDirection, includeSpentCoins],
  );

  useEffect(() => {
    updateCoins();

    const unlisten = events.syncEvent.listen((event) => {
      const type = event.payload.type;

      if (type === 'coin_state' || type === 'puzzle_batch_synced') {
        updateCoins();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [updateCoins]);

  const redownload = () => {
    if (!assetId) return;

    commands
      .resyncCat({ asset_id: assetId })
      .then(() => updateToken())
      .catch(addError);
  };

  const updateTokenDetails = async (updatedAsset: TokenRecord) => {
    return commands
      .updateCat({ record: updatedAsset })
      .then(() => updateToken())
      .catch(addError);
  };

  const setVisibility = (visible: boolean) => {
    if (!asset?.asset_id) return;
    const updatedAsset = { ...asset, visible };
    return updateTokenDetails(updatedAsset);
  };

  // Use ref to track current page to avoid dependency issues
  const currentPageRef = useRef(currentPage);
  currentPageRef.current = currentPage;

  // Reset to page 0 when sort parameters change
  useEffect(() => {
    setCurrentPage(0);
  }, [sortMode, sortDirection, includeSpentCoins]);

  // Update coins when page changes
  useEffect(() => {
    updateCoins(currentPage);
  }, [currentPage, updateCoins]);

  // Create the appropriate confirmation component based on the response
  const confirmationAdditionalData = useMemo(() => {
    if (!response?.additionalData?.content) return undefined;

    const content = response.additionalData.content;

    if (content.type === 'split') {
      return {
        title: t`Split Coins`,
        content: (
          <TokenConfirmation
            type='split'
            coins={content.coins}
            outputCount={content.outputCount || 2}
            ticker={content.ticker}
            precision={content.precision}
          />
        ),
      };
    } else if (content.type === 'combine') {
      return {
        title: t`Combine Coins`,
        content: (
          <TokenConfirmation
            type='combine'
            coins={content.coins}
            ticker={content.ticker}
            precision={content.precision}
          />
        ),
      };
    }

    return undefined;
  }, [response?.additionalData?.content]);

  // Get the appropriate handlers based on the asset type
  const splitHandler = useMemo(
    () => (!asset?.asset_id ? commands.splitXch : commands.splitCat),
    [asset?.asset_id],
  );

  const combineHandler = useMemo(
    () => (!asset?.asset_id ? commands.combineXch : commands.combineCat),
    [asset?.asset_id],
  );

  const autoCombineHandler = useMemo(
    () =>
      !asset?.asset_id
        ? commands.autoCombineXch
        : (...[req]: Parameters<typeof commands.autoCombineXch>) =>
            commands.autoCombineCat({
              ...req,
              asset_id: asset?.asset_id ?? '',
            }),
    [asset?.asset_id],
  );

  return (
    <>
      <Header
        title={
          <span>
            {asset
              ? getAssetDisplayName(asset.name, asset.ticker, 'token')
              : ''}{' '}
            {asset?.asset_id && (
              <>
                {' '}
                <span className='text-sm text-muted-foreground font-mono font-normal'>
                  {asset?.asset_id?.slice(0, 6) +
                    '...' +
                    asset?.asset_id?.slice(-4)}
                </span>{' '}
                <CopyButton
                  value={asset?.asset_id ?? ''}
                  className='w-4 h-4'
                  onCopy={() => {
                    toast.success(t`Asset ID copied to clipboard`);
                  }}
                />
              </>
            )}
          </span>
        }
      />
      {asset && (
        <Container>
          <div className='flex flex-col gap-4 max-w-screen-lg'>
            <TokenCard
              asset={asset}
              balanceInUsd={balanceInUsd}
              onRedownload={redownload}
              onVisibilityChange={() => {
                setVisibility(asset?.visible ?? true);
                navigate('/wallet');
              }}
              onUpdate={updateTokenDetails}
            />
            <OwnedCoinsCard
              coins={coins}
              asset={asset}
              splitHandler={splitHandler}
              combineHandler={combineHandler}
              autoCombineHandler={autoCombineHandler}
              setResponse={setResponse}
              selectedCoins={selectedCoins}
              setSelectedCoins={setSelectedCoins}
              currentPage={currentPage}
              totalCoins={totalCoins}
              pageSize={pageSize}
              setCurrentPage={setCurrentPage}
              sortMode={sortMode}
              sortDirection={sortDirection}
              includeSpentCoins={includeSpentCoins}
              onSortModeChange={setSortMode}
              onSortDirectionChange={setSortDirection}
              onIncludeSpentCoinsChange={setIncludeSpentCoins}
            />
          </div>
        </Container>
      )}

      <ConfirmationDialog
        response={response}
        showRecipientDetails={false}
        close={() => setResponse(null)}
        onConfirm={() => setSelectedCoins({})}
        additionalData={confirmationAdditionalData}
      />
    </>
  );
}
