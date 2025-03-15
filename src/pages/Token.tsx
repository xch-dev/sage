import ConfirmationDialog from '@/components/ConfirmationDialog';
import Container from '@/components/Container';
import { CopyButton } from '@/components/CopyButton';
import Header from '@/components/Header';
import { QRCodeDialog } from '@/components/QRCodeDialog';
import { useTokenManagement } from '@/hooks/useTokenManagement';
import { t } from '@lingui/core/macro';
import { useParams } from 'react-router-dom';
import { CoinsCard } from '@/components/CoinsCard';
import { TokenCard } from '@/components/TokenCard';
import { SplitTokenConfirmation } from '@/components/confirmations/SplitTokenConfirmation';
import { CombineTokenConfirmation } from '@/components/confirmations/CombineTokenConfirmation';
import { useMemo } from 'react';
import { commands } from '../bindings';

export default function Token() {
  const { asset_id: assetId } = useParams();
  const {
    asset,
    coins,
    precision,
    balanceInUsd,
    response,
    selectedCoins,
    isReceiveOpen,
    receive_address,
    setResponse,
    setSelectedCoins,
    setReceiveOpen,
    redownload,
    setVisibility,
    updateCatDetails,
  } = useTokenManagement(assetId);

  // Create the appropriate confirmation component based on the response
  const confirmationAdditionalData = useMemo(() => {
    if (!response?.additionalData?.content) return undefined;

    const content = response.additionalData.content;

    if (content.type === 'split') {
      return {
        title: t`Split Coins`,
        content: (
          <SplitTokenConfirmation
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
          <CombineTokenConfirmation
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
    () => (asset?.asset_id === 'xch' ? commands.splitXch : commands.splitCat),
    [asset?.asset_id],
  );

  const combineHandler = useMemo(
    () =>
      asset?.asset_id === 'xch' ? commands.combineXch : commands.combineCat,
    [asset?.asset_id],
  );

  return (
    <>
      <Header
        title={
          <span>
            {asset ? (asset.name ?? t`Unknown asset`) : ''}{' '}
            {asset?.asset_id !== 'xch' && (
              <CopyButton value={asset?.asset_id ?? ''} />
            )}
          </span>
        }
      />
      <Container>
        <div className='flex flex-col gap-4 max-w-screen-lg'>
          <TokenCard
            asset={asset}
            assetId={assetId}
            precision={precision}
            balanceInUsd={balanceInUsd}
            onReceiveClick={() => setReceiveOpen(true)}
            onRedownload={redownload}
            onVisibilityChange={setVisibility}
            onUpdateCat={updateCatDetails}
          />
          <CoinsCard
            precision={precision}
            coins={coins}
            asset={asset}
            splitHandler={splitHandler}
            combineHandler={combineHandler}
            setResponse={setResponse}
            selectedCoins={selectedCoins}
            setSelectedCoins={setSelectedCoins}
          />
        </div>
      </Container>

      <QRCodeDialog
        isOpen={isReceiveOpen}
        onClose={setReceiveOpen}
        asset={asset}
        receive_address={receive_address}
      />
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
