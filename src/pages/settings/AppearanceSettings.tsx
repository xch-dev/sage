import { ThemeSelectorSimple } from '@/components/ThemeSelector';
import { Button } from '@/components/ui/button';
import { resetViewPreferences } from '@/lib/viewPreferences';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { Link } from 'react-router-dom';
import { toast } from 'react-toastify';
import { SettingItem, SettingsSection } from './SettingsSection';

export function AppearanceSettings() {
  return (
    <>
      <SettingsSection
        title={t`Theme`}
        description={t`Choose your preferred theme`}
      >
        <ThemeSelectorSimple />
        <div className='p-3'>
          <Button variant='outline' size='sm' asChild>
            <Link to='/themes'>
              <Trans>More Themes...</Trans>
            </Link>
          </Button>
        </div>
      </SettingsSection>

      <SettingsSection title={t`Lists`}>
        <SettingItem
          label={t`View Preferences`}
          description={t`Reset sorting, filters, page sizes, card sizes and view modes to their defaults`}
          control={
            <Button
              variant='outline'
              size='sm'
              onClick={() => {
                resetViewPreferences();
                toast.success(t`View preferences reset`);
              }}
            >
              <Trans>Reset</Trans>
            </Button>
          }
        />
      </SettingsSection>
    </>
  );
}
