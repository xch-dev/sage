import { Amount, Asset, OptionRecord } from '@/bindings';
import { AssetIcon } from '@/components/AssetIcon';
import { NumberFormat } from '@/components/NumberFormat';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Separator } from '@/components/ui/separator';
import useOfferStateWithDefault from '@/hooks/useOfferStateWithDefault';
import { OptionActionHandlers } from '@/hooks/useOptionActions';
import { formatTimestamp, fromMojos } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { InfoCircledIcon } from '@radix-ui/react-icons';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import {
  AlertCircle,
  Copy,
  EyeIcon,
  EyeOff,
  FilePenLine,
  Flame,
  HandCoins,
  MoreVerticalIcon,
  SendIcon,
} from 'lucide-react';
import { Link, useNavigate } from 'react-router-dom';
import { toast } from 'react-toastify';
import { AddressItem } from './AddressItem';
import { LabeledItem } from './LabeledItem';

interface OptionCardProps {
  option: OptionRecord;
  actionHandlers: OptionActionHandlers;
}

export function OptionCard({ option, actionHandlers }: OptionCardProps) {
  const [offerState, setOfferState] = useOfferStateWithDefault();
  const navigate = useNavigate();

  return (
    <>
      <Card
        key={option.launcher_id}
        className={`${!option.visible ? 'opacity-50 grayscale' : option.created_height === null ? 'pulsate-opacity' : ''}`}
      >
        <CardHeader className='-mt-2 flex flex-row items-center justify-between space-y-0 pb-2 pr-2 space-x-2'>
          <CardTitle className='text-md font-medium truncate flex items-center'>
            <FilePenLine className='mr-2 h-4 w-4' aria-hidden='true' />
            <Link
              to={`/options/${option.launcher_id}`}
              className='hover:underline truncate'
            >
              {option.name}
            </Link>
          </CardTitle>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant='ghost' size='icon'>
                <MoreVerticalIcon className='h-5 w-5' aria-hidden='true' />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align='end'>
              <DropdownMenuGroup>
                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    actionHandlers.onExercise(option);
                  }}
                  disabled={option.created_height === null}
                >
                  <HandCoins className='mr-2 h-4 w-4' aria-hidden='true' />
                  <Trans>Exercise</Trans>
                </DropdownMenuItem>

                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    actionHandlers.onTransfer(option);
                  }}
                  disabled={option.created_height === null}
                >
                  <SendIcon className='mr-2 h-4 w-4' aria-hidden='true' />
                  <Trans>Transfer</Trans>
                </DropdownMenuItem>

                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    actionHandlers.onBurn(option);
                  }}
                  disabled={option.created_height === null}
                >
                  <Flame className='mr-2 h-4 w-4' />
                  <Trans>Burn</Trans>
                </DropdownMenuItem>

                <DropdownMenuSeparator />

                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();

                    const newOptions = [...offerState.offered.options];
                    newOptions.push(option.launcher_id);

                    setOfferState({
                      offered: {
                        ...offerState.offered,
                        options: newOptions,
                      },
                    });

                    toast.success(t`Click here to go to offer.`, {
                      onClick: () => navigate('/offers/make'),
                    });
                  }}
                  disabled={
                    option.created_height === null ||
                    option.expiration_seconds * 1000 < Date.now() ||
                    offerState.offered.options.includes(option.launcher_id)
                  }
                  aria-label={t`Add ${option.name || 'Unknown Option'} to offer`}
                >
                  <HandCoins className='mr-2 h-4 w-4' aria-hidden='true' />
                  <span>
                    <Trans>Add to Offer</Trans>
                  </span>
                </DropdownMenuItem>

                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    writeText(option.launcher_id);
                    toast.success(t`Option ID copied to clipboard`);
                  }}
                >
                  <Copy className='mr-2 h-4 w-4' aria-hidden='true' />
                  <Trans>Copy ID</Trans>
                </DropdownMenuItem>

                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    actionHandlers.onToggleVisibility(option);
                  }}
                  aria-label={
                    option.visible
                      ? t`Hide ${option.name || 'Unknown Option'}`
                      : t`Show ${option.name || 'Unknown Option'}`
                  }
                >
                  {option.visible ? (
                    <EyeOff className='mr-2 h-4 w-4' />
                  ) : (
                    <EyeIcon className='mr-2 h-4 w-4' />
                  )}
                  {option.visible ? t`Hide` : t`Show`}
                </DropdownMenuItem>
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>
        </CardHeader>
        <CardContent>
          <div className='flex flex-col gap-2'>
            <AddressItem label={t`Option ID`} address={option.launcher_id} />

            <LabeledItem
              label={t`Expiration`}
              content={formatTimestamp(option.expiration_seconds)}
            />

            {option.expiration_seconds * 1000 < Date.now() ? (
              <div className='flex items-center gap-1.5 text-sm font-medium text-red-500'>
                <AlertCircle className='h-4 w-4' aria-hidden='true' />
                <Trans>Expired</Trans>
              </div>
            ) : option.expiration_seconds * 1000 <
              Date.now() + 24 * 60 * 60 * 1000 ? (
              <div className='flex items-center gap-1.5 text-sm font-medium text-yellow-500'>
                <AlertCircle className='h-4 w-4' aria-hidden='true' />
                <Trans>Expiring soon</Trans>
              </div>
            ) : (
              <div className='flex items-center gap-1.5 text-sm font-medium text-blue-400'>
                <InfoCircledIcon className='h-4 w-4' aria-hidden='true' />
                <Trans>Active</Trans>
              </div>
            )}

            <Separator className='my-1' />

            <div className='flex flex-col gap-2'>
              <OptionAssetPreview
                asset={option.underlying_asset}
                amount={option.underlying_amount}
                option={option}
              />
              <OptionAssetPreview
                asset={option.strike_asset}
                amount={option.strike_amount}
                option={option}
              />
            </div>
          </div>
        </CardContent>
      </Card>
    </>
  );
}

interface OptionAssetPreviewProps {
  asset: Asset;
  amount: Amount;
  option: OptionRecord;
}

function OptionAssetPreview({
  asset,
  amount,
  option,
}: OptionAssetPreviewProps) {
  return (
    <LabeledItem
      label={
        asset === option.underlying_asset
          ? t`Underlying Asset`
          : t`Strike Price`
      }
      content={null}
    >
      <div className='flex items-center gap-2' key={asset.asset_id ?? 'xch'}>
        <AssetIcon asset={asset} size='md' />
        <div className='text-sm text-muted-foreground truncate'>
          {asset.kind === 'token' && (
            <NumberFormat
              value={fromMojos(amount, asset.precision)}
              minimumFractionDigits={0}
              maximumFractionDigits={asset.precision}
            />
          )}{' '}
          {asset.name ?? asset.ticker ?? t`Unknown`}
        </div>
      </div>
    </LabeledItem>
  );
}
