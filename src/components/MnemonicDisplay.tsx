import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { CopyIcon, RefreshCwIcon } from 'lucide-react';
import { useCallback } from 'react';
import { toast } from 'react-toastify';

interface MnemonicDisplayProps {
  mnemonic: string | undefined;
  label?: React.ReactNode;
  onRegenerate?: () => void;
}

export function MnemonicDisplay({
  mnemonic,
  label,
  onRegenerate,
}: MnemonicDisplayProps) {
  const copyMnemonic = useCallback(() => {
    if (!mnemonic) return;
    writeText(mnemonic);
    toast.success(t`Mnemonic copied to clipboard`);
  }, [mnemonic]);

  return (
    <div>
      <div className='flex justify-between items-center mb-2'>
        <Label>{label ?? <Trans>Mnemonic</Trans>}</Label>
        <div>
          {onRegenerate && (
            <Button
              type='button'
              variant='ghost'
              size='sm'
              onClick={onRegenerate}
            >
              <RefreshCwIcon className='h-4 w-4' />
            </Button>
          )}
          <Button
            type='button'
            variant='ghost'
            size='sm'
            onClick={copyMnemonic}
          >
            <CopyIcon className='h-4 w-4' />
          </Button>
        </div>
      </div>
      <div className='flex flex-wrap'>
        {mnemonic?.split(' ').map((word, i) => (
          <Badge
            // eslint-disable-next-line react/no-array-index-key
            key={`${word}-${i}`}
            variant='outline'
            className='py-1.5 px-2.5 m-0.5 rounded-lg font-medium'
          >
            {word}
          </Badge>
        ))}
      </div>
    </div>
  );
}
