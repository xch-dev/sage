import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { useLanguage } from '@/contexts/LanguageContext';
import { t } from '@lingui/core/macro';
import { SettingItem, SettingsSection } from './SettingsSection';

export function GeneralSettings() {
  const { locale, changeLanguage } = useLanguage();

  return (
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
  );
}
