import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { TimelockInput } from '@/lib/vaultRecovery';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { useId } from 'react';

interface VaultRecoveryFieldsProps {
  recoveryKey: string;
  onRecoveryKeyChange: (key: string) => void;
  timelock: TimelockInput;
  onTimelockChange: (timelock: TimelockInput) => void;
  /** Whether the recovery key can be left empty. */
  optional?: boolean;
}

/** The recovery key and timelock that rebuild the config of a vault without
 *  memos, from what its spends reveal. */
export function VaultRecoveryFields({
  recoveryKey,
  onRecoveryKeyChange,
  timelock,
  onTimelockChange,
  optional = false,
}: VaultRecoveryFieldsProps) {
  const id = useId();

  const parts = [
    ['days', t`Days`],
    ['hours', t`Hours`],
    ['minutes', t`Minutes`],
  ] as const;

  return (
    <div className='space-y-4'>
      <div className='space-y-2'>
        <Label htmlFor={`${id}-key`}>
          {optional ? (
            <Trans>Recovery Key (Optional)</Trans>
          ) : (
            <Trans>Recovery Key</Trans>
          )}
        </Label>
        <Textarea
          id={`${id}-key`}
          className='resize-none h-20 font-mono'
          value={recoveryKey}
          onChange={(event) => onRecoveryKeyChange(event.target.value)}
        />
        <p className='text-sm text-muted-foreground'>
          <Trans>
            The public key of the vault&apos;s recovery member, or the mnemonic
            or secret key of a BLS recovery key, which isn&apos;t saved. For a
            vault without memos on chain, its config is rebuilt from what its
            spends reveal.
          </Trans>
        </p>
      </div>

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
              value={timelock[part]}
              onChange={(event) =>
                onTimelockChange({ ...timelock, [part]: event.target.value })
              }
            />
          ))}
        </div>
        <p className='text-sm text-muted-foreground'>
          <Trans>
            If it&apos;s left empty, common timelocks are tried, which can take
            a few seconds.
          </Trans>
        </p>
      </div>
    </div>
  );
}
