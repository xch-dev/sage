import { CopyBox } from '@/components/CopyBox';
import { t } from '@lingui/core/macro';
import { useId } from 'react';
import { toast } from 'react-toastify';

export interface AddressItemProps {
  label: string;
  address: string;
  className?: string;
  hideLabel?: boolean;
  inputClassName?: string;
  truncateMiddle?: boolean;
}

export function AddressItem({
  label,
  address,
  className = '',
  hideLabel = false,
  inputClassName,
  truncateMiddle = false,
}: AddressItemProps) {
  const labelId = useId();
  const contentId = useId();

  // Don't render if address is empty
  if (!address || address.trim() === '') {
    return null;
  }

  return (
    <div
      className={className}
      role='region'
      aria-labelledby={labelId}
      aria-label={t`${label} address section`}
    >
      {!hideLabel && (
        <label
          id={labelId}
          htmlFor={contentId}
          className='text-sm font-medium text-muted-foreground block mb-1'
        >
          {label}
        </label>
      )}
      <CopyBox
        id={contentId}
        title={t`Copy ${label}: ${address}`}
        value={address}
        onCopy={() => toast.success(t`${label} copied to clipboard`)}
        aria-label={t`${label}: ${address} (click to copy)`}
        aria-describedby={labelId}
        inputClassName={inputClassName}
        truncateMiddle={truncateMiddle}
      />
    </div>
  );
}
