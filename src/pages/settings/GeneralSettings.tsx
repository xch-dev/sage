import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { useLanguage } from '@/contexts/LanguageContext';
import {
  FIAT_CURRENCIES,
  FiatCurrency,
  usePriceSettings,
} from '@/hooks/usePriceSettings';
import { t } from '@lingui/core/macro';
import { SettingItem, SettingsSection } from './SettingsSection';

export function GeneralSettings() {
  const { locale, changeLanguage } = useLanguage();
  const { currency, setCurrency, pricesEnabled, setPricesEnabled } =
    usePriceSettings();

  return (
    <>
      <SettingsSection title={t`Language`}>
        <SettingItem
          label={t`Language`}
          description={t`Choose your preferred language`}
          control={
            <Select value={locale} onValueChange={changeLanguage}>
              <SelectTrigger className='w-[140px]' aria-label={t`Language`}>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value='en-US'>English</SelectItem>
                <SelectItem value='de-DE'>Deutsch</SelectItem>
                <SelectItem value='zh-CN'>中文</SelectItem>
                <SelectItem value='es-MX'>Español</SelectItem>
              </SelectContent>
            </Select>
          }
        />
      </SettingsSection>

      <SettingsSection title={t`Prices`}>
        <SettingItem
          label={t`Price Lookups`}
          description={t`Look up token prices with CoinGecko and Dexie to show what your balances are worth`}
          control={
            <Switch
              checked={pricesEnabled}
              onCheckedChange={setPricesEnabled}
            />
          }
        />
        <SettingItem
          label={t`Currency`}
          description={t`The currency that values and prices are shown in`}
          control={
            <Select
              value={currency}
              disabled={!pricesEnabled}
              onValueChange={(value) => setCurrency(value as FiatCurrency)}
            >
              <SelectTrigger className='w-[140px]' aria-label={t`Currency`}>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {FIAT_CURRENCIES.map((code) => (
                  <SelectItem key={code} value={code}>
                    {code}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          }
        />
      </SettingsSection>
    </>
  );
}
