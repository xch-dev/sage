import { NumberFormat } from '@/components/NumberFormat';
import { QRCodeDialog } from '@/components/QRCodeDialog';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader } from '@/components/ui/card';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { fromMojos } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { openUrl } from '@tauri-apps/plugin-opener';
import {
  ExternalLink,
  Eye,
  EyeOff,
  HandHelping,
  MoreHorizontalIcon,
  Pencil,
  RefreshCw,
  Send,
} from 'lucide-react';
import { useState } from 'react';
import { Link } from 'react-router-dom';
import { toast } from 'react-toastify';
import { CatRecord } from '../bindings';
import { AssetIcon } from './AssetIcon';

interface TokenCardProps {
  asset: CatRecord | null;
  assetId: string | undefined;
  precision: number;
  balanceInUsd: string;
  onRedownload: () => void;
  onVisibilityChange: (visible: boolean) => void;
  onUpdateCat: (updatedAsset: CatRecord) => Promise<void>;
  receive_address: string;
}

export function TokenCard({
  asset,
  assetId,
  precision,
  balanceInUsd,
  onRedownload,
  onVisibilityChange,
  onUpdateCat,
  receive_address,
}: TokenCardProps) {
  const [isEditOpen, setIsEditOpen] = useState(false);
  const [isReceiveOpen, setIsReceiveOpen] = useState(false);
  const [newName, setNewName] = useState('');
  const [newTicker, setNewTicker] = useState('');

  const handleEditClick = () => {
    if (asset) {
      setNewName(asset.name || '');
      setNewTicker(asset.ticker || '');
    }
    setIsEditOpen(true);
  };

  const handleEdit = () => {
    if (!newName || !newTicker || !asset) return;

    const updatedAsset = { ...asset };
    updatedAsset.name = newName;
    updatedAsset.ticker = newTicker;

    onUpdateCat(updatedAsset).finally(() => setIsEditOpen(false));
  };

  return (
    <>
      <Card>
        <CardHeader className='flex flex-col pb-2'>
          <div className='flex flex-row justify-between items-center space-y-0 space-x-2'>
            <div className='flex text-xl sm:text-4xl font-medium font-mono truncate'>
              <span className='truncate'>
                <NumberFormat
                  value={fromMojos(asset?.balance ?? 0, precision)}
                  minimumFractionDigits={0}
                  maximumFractionDigits={precision}
                />
                &nbsp;
              </span>
              {asset?.ticker}
            </div>
            <div className='flex-shrink-0'>
              <AssetIcon iconUrl={asset?.icon_url} kind='token' size='md' />
            </div>
          </div>
          <div className='text-sm text-muted-foreground'>
            <NumberFormat
              value={balanceInUsd}
              style='currency'
              currency='USD'
              minimumFractionDigits={2}
              maximumFractionDigits={2}
            />
          </div>
        </CardHeader>
        <CardContent className='flex flex-col gap-2'>
          <ReceiveAddress className='mt-2' />

          <div className='flex gap-2 mt-2 flex-wrap'>
            <Link to={`/wallet/send/${assetId}`}>
              <Button>
                <Send className='mr-2 h-4 w-4' /> <Trans>Send</Trans>
              </Button>
            </Link>
            <Button variant={'outline'} onClick={() => setIsReceiveOpen(true)}>
              <HandHelping className='mr-2 h-4 w-4' />
              <Trans>Receive</Trans>
            </Button>
            {asset && assetId !== 'xch' && (
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button variant='outline' size='icon'>
                    <MoreHorizontalIcon className='h-4 w-4' />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent>
                  <DropdownMenuItem onClick={handleEditClick}>
                    <Pencil className='mr-2 h-4 w-4' aria-hidden='true' />
                    <Trans>Edit</Trans>
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={onRedownload}>
                    <RefreshCw className='mr-2 h-4 w-4' aria-hidden='true' />
                    <Trans>Refresh Info</Trans>
                  </DropdownMenuItem>
                  <DropdownMenuItem
                    onClick={() => onVisibilityChange(!asset.visible)}
                  >
                    {asset.visible ? (
                      <EyeOff className='mr-2 h-4 w-4' aria-hidden='true' />
                    ) : (
                      <Eye className='mr-2 h-4 w-4' aria-hidden='true' />
                    )}
                    {asset.visible ? t`Hide` : t`Show`} {t`Asset`}
                  </DropdownMenuItem>
                  <DropdownMenuItem
                    onClick={() => {
                      openUrl(
                        `https://dexie.space/offers/XCH/${asset.asset_id}`,
                      ).catch((error) => {
                        toast.error(t`Failed to open dexie.space: ${error}`);
                      });
                    }}
                  >
                    <ExternalLink className='mr-2 h-4 w-4' aria-hidden='true' />
                    <Trans>View on dexie</Trans>
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            )}
          </div>
        </CardContent>
      </Card>

      <Dialog
        open={isEditOpen}
        onOpenChange={(open) => !open && setIsEditOpen(false)}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Trans>Edit Token Details</Trans>
            </DialogTitle>
            <DialogDescription>
              <Trans>Enter the new display details for this token</Trans>
            </DialogDescription>
          </DialogHeader>
          <div className='grid w-full items-center gap-4'>
            <div className='flex flex-col space-y-1.5'>
              <Label htmlFor='name'>
                <Trans>Name</Trans>
              </Label>
              <Input
                id='name'
                placeholder={t`Name of this token`}
                value={newName}
                onChange={(event) => setNewName(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key === 'Enter') {
                    event.preventDefault();
                    handleEdit();
                  }
                }}
              />
            </div>
          </div>
          <div className='grid w-full items-center gap-4'>
            <div className='flex flex-col space-y-1.5'>
              <Label htmlFor='ticker'>
                <Trans>Ticker</Trans>
              </Label>
              <Input
                id='ticker'
                placeholder={t`Ticker for this token`}
                value={newTicker}
                onChange={(event) => setNewTicker(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key === 'Enter') {
                    event.preventDefault();
                    handleEdit();
                  }
                }}
              />
            </div>
          </div>

          <DialogFooter className='gap-2'>
            <Button
              variant='outline'
              onClick={() => {
                setIsEditOpen(false);
                setNewName('');
                setNewTicker('');
              }}
            >
              <Trans>Cancel</Trans>
            </Button>
            <Button onClick={handleEdit} disabled={!newName || !newTicker}>
              <Trans>Rename</Trans>
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <QRCodeDialog
        isOpen={isReceiveOpen}
        onClose={() => setIsReceiveOpen(false)}
        asset={asset}
        qr_code_contents={receive_address}
      />
    </>
  );
}
