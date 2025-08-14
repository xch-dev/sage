import { Amount, Asset, OptionRecord } from '@/bindings';
import { AssetIcon } from '@/components/AssetIcon';
import { CopyBox } from '@/components/CopyBox';
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
import {
  OptionActionHandlers,
  useOptionActions,
} from '@/hooks/useOptionActions';
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
import { Link } from 'react-router-dom';
import { toast } from 'react-toastify';

interface OptionGridViewProps {
  options: OptionRecord[];
  updateOptions: () => void;
  showHidden: boolean;
}

export function OptionGridView({
  options,
  updateOptions,
  showHidden,
}: OptionGridViewProps) {
  const { actionHandlers, dialogs } = useOptionActions(updateOptions);

  const visibleOptions = showHidden
    ? options
    : options.filter((option) => option.visible);

  return (
    <>
      <div className='mt-4 grid gap-4 md:gap-4 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4'>
        {visibleOptions.map((option) => (
          <OptionCard
            key={option.launcher_id}
            option={option}
            actionHandlers={actionHandlers}
          />
        ))}
      </div>
      {dialogs}
    </>
  );
}

interface OptionCardProps {
  option: OptionRecord;
  actionHandlers: OptionActionHandlers;
}

function OptionCard({ option, actionHandlers }: OptionCardProps) {
  return (
    <>
      <Card
        key={option.launcher_id}
        className={`${!option.visible ? 'opacity-50 grayscale' : option.created_height === null ? 'pulsate-opacity' : ''}`}
      >
        <CardHeader className='-mt-2 flex flex-row items-center justify-between space-y-0 pb-2 pr-2 space-x-2'>
          <CardTitle className='text-md font-medium truncate flex items-center'>
            <FilePenLine className='mr-2 h-4 w-4' />
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
                <MoreVerticalIcon className='h-5 w-5' />
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
                  <HandCoins className='mr-2 h-4 w-4' />
                  <span>
                    <Trans>Exercise</Trans>
                  </span>
                </DropdownMenuItem>

                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    actionHandlers.onTransfer(option);
                  }}
                  disabled={option.created_height === null}
                >
                  <SendIcon className='mr-2 h-4 w-4' />
                  <span>
                    <Trans>Transfer</Trans>
                  </span>
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
                  <span>
                    <Trans>Burn</Trans>
                  </span>
                </DropdownMenuItem>

                <DropdownMenuSeparator />

                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    writeText(option.launcher_id);
                    toast.success(t`Option ID copied to clipboard`);
                  }}
                >
                  <Copy className='mr-2 h-4 w-4' />
                  <span>
                    <Trans>Copy ID</Trans>
                  </span>
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
                  <span>{option.visible ? t`Hide` : t`Show`}</span>
                </DropdownMenuItem>
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>
        </CardHeader>
        <CardContent>
          <div className='flex flex-col gap-2'>
            <div className='flex flex-col gap-1'>
              <div className='text-sm font-medium text-muted-foreground'>
                Option ID
              </div>
              <CopyBox value={option.launcher_id} title={t`Option ID`} />
            </div>

            <div className='flex flex-col gap-1'>
              <div className='text-sm font-medium text-muted-foreground'>
                Expiration
              </div>
              <div className='text-sm font-medium'>
                {formatTimestamp(option.expiration_seconds)}
              </div>
            </div>

            {option.expiration_seconds * 1000 < Date.now() ? (
              <div className='flex items-center gap-1.5 text-sm font-medium text-red-500'>
                <AlertCircle className='h-4 w-4' />
                <Trans>Expired</Trans>
              </div>
            ) : option.expiration_seconds * 1000 <
              Date.now() + 24 * 60 * 60 * 1000 ? (
              <div className='flex items-center gap-1.5 text-sm font-medium text-yellow-500'>
                <AlertCircle className='h-4 w-4' />
                <Trans>Expiring soon</Trans>
              </div>
            ) : (
              <div className='flex items-center gap-1.5 text-sm font-medium text-blue-400'>
                <InfoCircledIcon className='h-4 w-4' />
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
    <div className='flex flex-col gap-1'>
      <div className='text-sm font-medium text-muted-foreground'>
        {asset === option.underlying_asset
          ? t`Underlying Asset`
          : t`Strike Price`}
      </div>
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
    </div>
  );
}
