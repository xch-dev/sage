import { useEffect, useState } from 'react';
import { Input } from '@/components/ui/input';
import { cn } from '@/lib/utils';
import { isValidXchName, resolveXchName } from '@/utils/namesdao';

export interface AddressInputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  onResolvedAddress?: (address: string | null) => void;
}

export function AddressInput({ 
  value = '', 
  onChange,
  onResolvedAddress,
  className,
  ...props
}: AddressInputProps) {
  const [resolvedAddress, setResolvedAddress] = useState<string | null>(null);
  const [isResolving, setIsResolving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let mounted = true;

    async function resolveAddress() {
      // Trim whitespace from value
      const trimmedValue = value.toString().trim();
      
      if (!trimmedValue || !isValidXchName(trimmedValue)) {
        setResolvedAddress(null);
        setError(null);
        onResolvedAddress?.(null);
        return;
      }

      setIsResolving(true);
      setError(null);
      
      try {
        const address = await resolveXchName(trimmedValue);
        if (!mounted) return;

        setResolvedAddress(address);
        onResolvedAddress?.(address);
        
        if (!address) {
          setError('Invalid .xch name');
        }
      } catch (e) {
        if (!mounted) return;
        setError('Failed to resolve name');
        setResolvedAddress(null);
        onResolvedAddress?.(null);
      } finally {
        if (mounted) {
          setIsResolving(false);
        }
      }
    }

    resolveAddress();
    return () => { mounted = false; };
  }, [value, onResolvedAddress]);

  return (
    <div className="space-y-2">
      <Input 
        value={value}
        onChange={(e) => {
          // Trim whitespace on change
          const trimmedValue = e.target.value.trim();
          onChange?.({ ...e, target: { ...e.target, value: trimmedValue } });
        }}
        placeholder="Enter address or .xch name"
        className={cn(
          error && 'border-red-500 focus-visible:ring-red-500',
          className
        )}
        {...props}
      />
      {isResolving && (
        <div className="text-sm text-neutral-500 animate-pulse">
          Resolving name...
        </div>
      )}
      {error && (
        <div className="text-sm text-red-500">
          {error}
        </div>
      )}
      {resolvedAddress && (
        <div className="text-sm text-neutral-600 dark:text-neutral-400 break-all">
          Resolves to: {resolvedAddress}
        </div>
      )}
    </div>
  );
}
