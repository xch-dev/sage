import { ClawbackCoinsCard } from '@/components/ClawbackCoinsCard';
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
import { useEffect, useMemo, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { toast } from 'react-toastify';
import {
  CoinRecord,
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
  const [selectedOwnedCoins, setSelectedOwnedCoins] =
    useState<RowSelectionState>({});
  const [selectedClawbackCoins, setSelectedClawbackCoins] =
    useState<RowSelectionState>({});

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
    } else if (content.type === 'clawback') {
      return {
        title: t`Claw Back Details`,
        content: (
          <TokenConfirmation
            type='clawback'
            coins={content.coins}
            ticker={content.ticker}
            precision={content.precision}
          />
        ),
      };
    }

    return undefined;
  }, [response?.additionalData?.content]);

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
        <>
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
                asset={asset}
                setResponse={setResponse}
                selectedCoins={selectedOwnedCoins}
                setSelectedCoins={setSelectedOwnedCoins}
              />
              <ClawbackCoinsCard
                asset={asset}
                setResponse={setResponse}
                selectedCoins={selectedClawbackCoins}
                setSelectedCoins={setSelectedClawbackCoins}
              />
            </div>
          </Container>
        </>
      )}

      <ConfirmationDialog
        response={response}
        showRecipientDetails={false}
        close={() => setResponse(null)}
        onConfirm={() => {
          setSelectedOwnedCoins({});
          setSelectedClawbackCoins({});
        }}
        additionalData={confirmationAdditionalData}
      />
    </>
  );
}
