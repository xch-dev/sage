import { CopyBox } from '@/components/CopyBox';
import { t } from '@lingui/core/macro';
import { toast } from 'react-toastify';

export interface AddressItemProps {
  label: string;
  address: string;
}

export function AddressItem({ label, address }: AddressItemProps) {
  return (
    <div>
      <h6 className='text-sm font-semibold text-muted-foreground mb-2 '>
        {label}
      </h6>
      <CopyBox
        title={label}
        value={address}
        onCopy={() => toast.success(t`${label} copied to clipboard`)}
      />
    </div>
  );
}
