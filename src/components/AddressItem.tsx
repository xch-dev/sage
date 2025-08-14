import { CopyBox } from '@/components/CopyBox';
import { t } from '@lingui/core/macro';
import { toast } from 'react-toastify';

export interface AddressItemProps {
  label: string | null;
  address: string;
  className?: string;
}

export function AddressItem({
  label,
  address,
  className = '',
}: AddressItemProps) {
  return (
    <div className={className}>
      <h6 className='text-sm font-semibold text-muted-foreground mb-2'>
        {label}
      </h6>
      <CopyBox
        title={label ?? ''}
        value={address}
        onCopy={() => toast.success(t`${label} copied to clipboard`)}
      />
    </div>
  );
}
