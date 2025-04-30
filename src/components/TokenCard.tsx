import { Card, CardHeader, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { NumberFormat } from '@/components/NumberFormat';
import { fromMojos } from '@/lib/utils';
import { Link } from 'react-router-dom';
import { Trans } from '@lingui/react/macro';
import { t } from '@lingui/core/macro';
import { QRCodeDialog } from '@/components/QRCodeDialog';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import {
  Send,
  HandHelping,
  MoreHorizontalIcon,
  Pencil,
  RefreshCw,
  Eye,
  EyeOff,
  ExternalLink,
} from 'lucide-react';
import { openUrl } from '@tauri-apps/plugin-opener';
import { toast } from 'react-toastify';
import { CatRecord } from '../bindings';
import { useState } from 'react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';

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
  const [isEditOpen, setEditOpen] = useState(false);
  const [isReceiveOpen, setReceiveOpen] = useState(false);
  const [newName, setNewName] = useState('');
  const [newTicker, setNewTicker] = useState('');

  const handleEditClick = () => {
    if (asset) {
      setNewName(asset.name || '');
      setNewTicker(asset.ticker || '');
    }
    setEditOpen(true);
  };

  const handleEdit = () => {
    if (!newName || !newTicker || !asset) return;

    const updatedAsset = { ...asset };
    updatedAsset.name = newName;
    updatedAsset.ticker = newTicker;

    onUpdateCat(updatedAsset).finally(() => setEditOpen(false));
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
              <img
                alt='asset icon'
                src={asset?.icon_url ?? ''}
                className='h-8 w-8'
              />
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
            <Button variant={'outline'} onClick={() => setReceiveOpen(true)}>
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
                        console.error('Failed to open dexie.space:', error);
                        toast.error(t`Failed to open dexie.space`);
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
        onOpenChange={(open) => !open && setEditOpen(false)}
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
                setEditOpen(false);
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
        onClose={() => setReceiveOpen(false)}
        asset={asset}
        qr_code_contents={receive_address}
      />
    </>
  );
}
