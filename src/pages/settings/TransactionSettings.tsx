import { Switch } from '@/components/ui/switch';
import { FeeAmountInput } from '@/components/ui/masked-input';
import { useColdWalletUnsigned } from '@/hooks/useColdWalletUnsigned';
import { useDefaultClawback } from '@/hooks/useDefaultClawback';
import { useDefaultFee } from '@/hooks/useDefaultFee';
import { useDefaultOfferExpiry } from '@/hooks/useDefaultOfferExpiry';
import { useDefaultOfferSplit } from '@/hooks/useDefaultOfferSplit';
import { useErrors } from '@/hooks/useErrors';
import {
  Duration,
  durationParts,
  expirationDurationSeconds,
} from '@/lib/expiration';
import { offersEnabled } from '@/lib/features';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { useEffect, useState } from 'react';
import { commands, WalletDefaults } from '../../bindings';
import { SettingItem, SettingsSection, TimeInput } from './SettingsSection';

// Matches `DEFAULT_TRANSACTION_EXPIRATION` in `sage-config`, and is used when the
// setting is switched back on without a duration.
const DEFAULT_TRANSACTION_EXPIRATION = 3600;

// Matches `MIN_TRANSACTION_EXPIRATION` in `sage-config`.
const MIN_TRANSACTION_EXPIRATION = 5 * 60;

export function TransactionSettings() {
  const { addError } = useErrors();
  const { expiry, setExpiry } = useDefaultOfferExpiry();
  const { clawback, setClawback } = useDefaultClawback();
  const { splitByDefault, setSplitByDefault } = useDefaultOfferSplit();
  const { setFee } = useDefaultFee();

  const [defaultWalletConfig, setDefaultWalletConfig] =
    useState<WalletDefaults | null>(null);

  const [localExpiration, setLocalExpiration] = useState<Duration>(
    durationParts(DEFAULT_TRANSACTION_EXPIRATION),
  );

  useEffect(() => {
    commands
      .defaultWalletConfig()
      .then((defaults) => {
        setDefaultWalletConfig(defaults);
        if (defaults.transaction_expiration) {
          setLocalExpiration(durationParts(defaults.transaction_expiration));
        }
      })
      .catch(addError);
  }, [addError]);

  const transactionExpiration = defaultWalletConfig?.transaction_expiration;

  const setTransactionExpiration = (seconds: number) => {
    commands
      .setTransactionExpiration({ seconds })
      .then(() =>
        setDefaultWalletConfig((current) =>
          current ? { ...current, transaction_expiration: seconds } : current,
        ),
      )
      .catch(addError);
  };

  const changeLocalExpiration = (duration: Duration) => {
    setLocalExpiration(duration);

    // Shorter durations aren't allowed, and zero would turn the expiration off, which is
    // done with the switch instead.
    const seconds = expirationDurationSeconds(duration);
    if (seconds >= MIN_TRANSACTION_EXPIRATION)
      setTransactionExpiration(seconds);
  };

  const expirationTooShort =
    expirationDurationSeconds(localExpiration) < MIN_TRANSACTION_EXPIRATION;

  // A duration that's too short wasn't saved, so it goes back to the saved one.
  const restoreSavedExpiration = () => {
    if (!expirationTooShort) return;
    if (transactionExpiration) {
      setLocalExpiration(durationParts(transactionExpiration));
    }
  };

  return (
    <>
      <SettingsSection title={t`Defaults`}>
        <SettingItem
          label={t`Default Fee`}
          description={t`The fee new transactions start with`}
          control={
            // A fixed width, so the input doesn't shrink beside the description or change size
            // between modes.
            <FeeAmountInput
              wrapperClassName='w-64'
              onValueChange={(values) =>
                setFee(values.value === '' ? '0' : values.value)
              }
            />
          }
        />
        <SettingItem
          label={t`Default Clawback`}
          description={t`Set a default clawback time for transactions`}
          control={
            <Switch
              checked={clawback.enabled}
              onCheckedChange={(checked) => {
                setClawback({
                  ...clawback,
                  enabled: checked,
                });
              }}
            />
          }
        >
          {clawback.enabled && (
            <div className='grid grid-cols-3 gap-4 mt-2'>
              <TimeInput
                label={t`Days`}
                value={clawback.days}
                onChange={(value) => setClawback({ ...clawback, days: value })}
              />
              <TimeInput
                label={t`Hours`}
                value={clawback.hours}
                onChange={(value) => setClawback({ ...clawback, hours: value })}
              />
              <TimeInput
                label={t`Minutes`}
                value={clawback.minutes}
                onChange={(value) =>
                  setClawback({ ...clawback, minutes: value })
                }
              />
            </div>
          )}
        </SettingItem>
        {offersEnabled && (
          <SettingItem
            label={t`Default Offer Expiry`}
            description={t`Set a default expiration time for new offers`}
            control={
              <Switch
                checked={expiry.enabled}
                onCheckedChange={(checked) => {
                  setExpiry({
                    ...expiry,
                    enabled: checked,
                  });
                }}
              />
            }
          >
            {expiry.enabled && (
              <div className='grid grid-cols-3 gap-4 mt-2'>
                <TimeInput
                  label={t`Days`}
                  value={expiry.days}
                  onChange={(value) => setExpiry({ ...expiry, days: value })}
                />
                <TimeInput
                  label={t`Hours`}
                  value={expiry.hours}
                  onChange={(value) => setExpiry({ ...expiry, hours: value })}
                />
                <TimeInput
                  label={t`Minutes`}
                  value={expiry.minutes}
                  onChange={(value) => setExpiry({ ...expiry, minutes: value })}
                />
              </div>
            )}
          </SettingItem>
        )}
        {offersEnabled && (
          <SettingItem
            label={t`Split Coins for Offers`}
            description={t`Split coins to the exact amounts before making an offer, so the rest of your balance stays spendable. This can still be changed for each offer.`}
            control={
              <Switch
                checked={splitByDefault}
                onCheckedChange={setSplitByDefault}
              />
            }
          />
        )}
        <SettingItem
          label={t`Transaction Expiration`}
          description={t`Transactions that aren't confirmed in time are dropped. This doesn't apply to offers.`}
          control={
            <Switch
              checked={(transactionExpiration ?? 0) > 0}
              disabled={!defaultWalletConfig}
              onCheckedChange={(checked) => {
                if (checked) {
                  const seconds = expirationTooShort
                    ? DEFAULT_TRANSACTION_EXPIRATION
                    : expirationDurationSeconds(localExpiration);
                  setLocalExpiration(durationParts(seconds));
                  setTransactionExpiration(seconds);
                } else {
                  setTransactionExpiration(0);
                }
              }}
            />
          }
        >
          {(transactionExpiration ?? 0) > 0 && (
            <div
              className='grid grid-cols-3 gap-4 mt-2'
              onBlur={(event) => {
                // Moving between the fields isn't leaving them.
                if (!event.currentTarget.contains(event.relatedTarget)) {
                  restoreSavedExpiration();
                }
              }}
            >
              <TimeInput
                label={t`Days`}
                value={localExpiration.days}
                onChange={(value) =>
                  changeLocalExpiration({ ...localExpiration, days: value })
                }
              />
              <TimeInput
                label={t`Hours`}
                value={localExpiration.hours}
                onChange={(value) =>
                  changeLocalExpiration({ ...localExpiration, hours: value })
                }
              />
              <TimeInput
                label={t`Minutes`}
                value={localExpiration.minutes}
                onChange={(value) =>
                  changeLocalExpiration({ ...localExpiration, minutes: value })
                }
              />
              {expirationTooShort && (
                <p className='col-span-3 text-sm text-destructive'>
                  <Trans>The expiration must be at least 5 minutes</Trans>
                </p>
              )}
            </div>
          )}
        </SettingItem>
      </SettingsSection>

      <ColdWalletSettings />
    </>
  );
}

function ColdWalletSettings() {
  const { allowUnsigned, setAllowUnsigned } = useColdWalletUnsigned();

  return (
    <SettingsSection title={t`Cold Wallets`}>
      <SettingItem
        label={t`Allow Unsigned Transactions`}
        description={t`Let cold wallets build transactions and export the unsigned spend bundle to sign elsewhere`}
        control={
          <Switch checked={allowUnsigned} onCheckedChange={setAllowUnsigned} />
        }
      />
    </SettingsSection>
  );
}
