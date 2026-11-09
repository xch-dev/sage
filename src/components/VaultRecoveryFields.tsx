import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { RecoveryInput, recoveryKeys } from '@/lib/vaultRecovery';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { useId } from 'react';

interface VaultRecoveryFieldsProps {
  value: RecoveryInput;
  onChange: (value: RecoveryInput) => void;
}

/** The recovery keys, threshold, and timelock that rebuild the config of a
 *  vault without memos, from what its spends reveal. Without keys, the vault
 *  is rebuilt as one without recovery. */
export function VaultRecoveryFields({
  value,
  onChange,
}: VaultRecoveryFieldsProps) {
  const id = useId();
  const keyCount = recoveryKeys(value.keys).length;

  const parts = [
    ['days', t`Days`],
    ['hours', t`Hours`],
    ['minutes', t`Minutes`],
  ] as const;

  return (
    <div className='space-y-4'>
      <div className='space-y-2'>
        <Label htmlFor={`${id}-keys`}>
          <Trans>Recovery Keys (Optional)</Trans>
        </Label>
        <Textarea
          id={`${id}-keys`}
          className='resize-none h-20 font-mono'
          value={value.keys}
          onChange={(event) => onChange({ ...value, keys: event.target.value })}
        />
        <p className='text-sm text-muted-foreground'>
          <Trans>
            One per line, the key of each of the vault&apos;s recovery members:
            a public key, the mnemonic or secret key of a BLS key, which
            isn&apos;t saved, or the vault1 ID of a vault. For a vault without
            memos on chain, its config is rebuilt from what its spends reveal.
            Leave it empty if the vault has no recovery.
          </Trans>
        </p>
      </div>

      {keyCount > 0 && (
        <>
          {keyCount > 1 && (
            <div className='space-y-2'>
              <Label htmlFor={`${id}-threshold`}>
                <Trans>Recovery Threshold (Optional)</Trans>
              </Label>
              <Input
                id={`${id}-threshold`}
                inputMode='numeric'
                value={value.threshold}
                onChange={(event) =>
                  onChange({ ...value, threshold: event.target.value })
                }
              />
              <p className='text-sm text-muted-foreground'>
                <Trans>
                  How many of the {keyCount} recovery keys must sign. If
                  it&apos;s left empty, every threshold is tried.
                </Trans>
              </p>
            </div>
          )}

          <div className='space-y-2'>
            <Label>
              <Trans>Recovery Timelock (Optional)</Trans>
            </Label>
            <div className='grid grid-cols-3 gap-2'>
              {parts.map(([part, label]) => (
                <Input
                  key={part}
                  inputMode='numeric'
                  placeholder={label}
                  aria-label={label}
                  value={value.timelock[part]}
                  onChange={(event) =>
                    onChange({
                      ...value,
                      timelock: {
                        ...value.timelock,
                        [part]: event.target.value,
                      },
                    })
                  }
                />
              ))}
            </div>
            <p className='text-sm text-muted-foreground'>
              <Trans>
                If it&apos;s left empty, common timelocks are tried, which can
                take a few seconds, or longer with several keys.
              </Trans>
            </p>
          </div>
        </>
      )}
    </div>
  );
}
