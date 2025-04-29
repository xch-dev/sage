import StyledQRCode from '@/components/StyledQrCode';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Trans } from '@lingui/react/macro';
import { useEffect, useState } from 'react';
import { CatRecord } from '../bindings';
import { openUrl } from '@tauri-apps/plugin-opener';
import { isValidUrl } from '@/lib/utils';

interface QRCodeDialogProps {
  isOpen: boolean;
  onClose: (open: boolean) => void;
  asset: CatRecord | null;
  qr_code_contents: string;
  title?: string;
  description?: string;
}

const getImageDataUrl = async (url: string) => {
  try {
    const response = await fetch(url);
    const blob = await response.blob();
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onloadend = () => resolve(reader.result);
      reader.onerror = reject;
      reader.readAsDataURL(blob);
    });
  } catch (error) {
    console.error('Failed to load image:', error);
    return null;
  }
};

const QRCodeCopyButton = ({
  qr_code_contents,
}: {
  qr_code_contents: string;
}) => {
  const [copySuccess, setCopySuccess] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(qr_code_contents);
      setCopySuccess(true);
      setTimeout(() => setCopySuccess(false), 2000); // Reset after 2 seconds
    } catch (err) {
      console.error('Failed to copy text:', err);
    }
  };

  return (
    <Button
      size='lg'
      variant='secondary'
      onClick={handleCopy}
      className='w-full'
    >
      {copySuccess ? <Trans>Copied!</Trans> : <Trans>Copy</Trans>}
    </Button>
  );
};

export function QRCodeDialog({
  isOpen,
  onClose,
  asset,
  qr_code_contents,
  title,
  description,
}: QRCodeDialogProps) {
  const [imageDataUrl, setImageDataUrl] = useState<string | undefined>(
    undefined,
  );

  useEffect(() => {
    if (asset?.icon_url) {
      getImageDataUrl(asset.icon_url)
        .then((dataUrl) => setImageDataUrl(dataUrl as string))
        .catch((error) => {
          console.error('Failed to load image:', error);
        });
    }
  }, [asset?.icon_url]);

  const ticker = asset?.ticker || '';
  const name = asset?.name || '';

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className='sm:max-w-md'>
        <DialogHeader>
          <DialogTitle>{title || <Trans>Receive {ticker}</Trans>}</DialogTitle>
          <DialogDescription>
            {description || <Trans>Use this address to receive {name}</Trans>}
          </DialogDescription>
        </DialogHeader>
        <div className='flex'>
          <div className='flex flex-col items-center justify-center'>
            <div className='py-4'>
              <StyledQRCode
                data={qr_code_contents}
                cornersSquareOptions={{
                  type: 'extra-rounded',
                }}
                dotsOptions={{
                  type: 'rounded',
                  color: '#000000',
                }}
                backgroundOptions={{}}
                image={imageDataUrl}
                imageOptions={{
                  hideBackgroundDots: true,
                  imageSize: 0.4,
                  margin: 5,
                  saveAsBlob: true,
                }}
              />
            </div>
            <span className='text-center break-words break-all'>
              {isValidUrl(qr_code_contents) ? (
                <a
                  href='#'
                  onClick={(e) => {
                    e.preventDefault();
                    openUrl(qr_code_contents);
                  }}
                  className='text-blue-600 hover:underline'
                >
                  {qr_code_contents}
                </a>
              ) : (
                qr_code_contents
              )}
            </span>
            <div className='pt-8 w-full'>
              <QRCodeCopyButton qr_code_contents={qr_code_contents} />
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
