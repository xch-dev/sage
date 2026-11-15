import { ThemeSelectorSimple } from '@/components/ThemeSelector';
import { Button } from '@/components/ui/button';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { Link } from 'react-router-dom';
import { SettingsSection } from './SettingsSection';

export function AppearanceSettings() {
  return (
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
  );
}
