import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { CopyButton } from '@/components/CopyButton';
import { toast } from 'react-toastify';
import { ConfirmationCard } from './ConfirmationCard';

interface SendTokenConfirmationProps {
  currentMemo: string;
}

export function SendTokenConfirmation({
  currentMemo,
}: SendTokenConfirmationProps) {
  return (
    <div className='space-y-3 text-xs'>
      <ConfirmationCard title={<Trans>Memo</Trans>}>
        <div className='flex items-center justify-between'>
          <div className='break-words whitespace-pre-wrap flex-1'>
            {currentMemo}
          </div>
          <CopyButton
            value={currentMemo}
            className='h-4 w-4 shrink-0 ml-2'
            onCopy={() => toast.success(t`Data copied to clipboard`)}
          />
        </div>
      </ConfirmationCard>
    </div>
  );
}
