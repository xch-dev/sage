import { CoinsCard } from '@/components/CoinsCard';
import ConfirmationDialog from '@/components/ConfirmationDialog';
import Container from '@/components/Container';
import { CopyButton } from '@/components/CopyButton';
import Header from '@/components/Header';
import { TokenCard } from '@/components/TokenCard';
import { TokenConfirmation } from '@/components/confirmations/TokenConfirmation';
import { useTokenState } from '@/hooks/useTokenState';
import { t } from '@lingui/core/macro';
import { useMemo } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { toast } from 'react-toastify';
import { commands } from '../bindings';
import { getAssetDisplayName } from '@/lib/utils';

export default function Token() {
  const { asset_id: assetId } = useParams();
  const {
    asset,
    coins,
    balanceInUsd,
    response,
    selectedCoins,
    receive_address,
    currentPage,
    totalCoins,
    pageSize,
    sortMode,
    sortDirection,
    includeSpentCoins,
    setResponse,
    setSelectedCoins,
    setCurrentPage,
    setSortMode,
    setSortDirection,
    setIncludeSpentCoins,
    redownload,
    setVisibility,
    updateCatDetails,
  } = useTokenState(assetId);
  const navigate = useNavigate();

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
    () => (asset?.is_xch ? commands.splitXch : commands.splitCat),
    [asset?.is_xch],
  );

  const combineHandler = useMemo(
    () => (asset?.is_xch ? commands.combineXch : commands.combineCat),
    [asset?.is_xch],
  );

  const autoCombineHandler = useMemo(
    () =>
      asset?.is_xch
        ? commands.autoCombineXch
        : (...[req]: Parameters<typeof commands.autoCombineXch>) =>
            commands.autoCombineCat({
              ...req,
              asset_id: asset?.asset_id ?? '',
            }),
    [asset?.is_xch],
  );

  return (
    <>
      <Header
        title={
          <span>
            {asset ? getAssetDisplayName(asset.name, asset.ticker, 'token') : ''}{' '}
            {!asset?.is_xch && (
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
            onUpdateCat={updateCatDetails}
            receive_address={receive_address}
          />
          <CoinsCard
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
