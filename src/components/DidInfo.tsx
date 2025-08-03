import { CopyBox } from '@/components/CopyBox';
import ProfileCard from '@/components/ProfileCard';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { toast } from 'react-toastify';

export interface DidInfoProps {
  did?: string | null;
  title: string;
  className?: string;
}

export function DidInfo({ did, title, className = '' }: DidInfoProps) {
  if (!did) {
    return (
      <div className={className}>
        <h6 className='text-md font-bold'>
          <Trans>{title}</Trans>
        </h6>
        <CopyBox
          title={t`DID`}
          value={t`None`}
          onCopy={() => toast.success(t`DID copied to clipboard`)}
        />
      </div>
    );
  }

  return (
    <div className={className}>
      <h6 className='text-md font-bold'>
        <Trans>{title}</Trans>
      </h6>
      <CopyBox
        title={t`DID`}
        value={did}
        onCopy={() => toast.success(t`DID copied to clipboard`)}
      />
      <div className='mt-1'>
        <ProfileCard did={did} variant='compact' />
      </div>
    </div>
  );
}
