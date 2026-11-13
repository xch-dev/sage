import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Trans } from '@lingui/react/macro';
import { Link2 } from 'lucide-react';
import { useLocation } from 'react-router-dom';

/** Navigation state marking a page opened by a `sage:` link. */
export const DEEP_LINK_NAV_STATE = { fromDeepLink: true } as const;

/** Warns that the page's details came from a link rather than the user. */
export function DeepLinkNotice({ className }: { className?: string }) {
  const { state } = useLocation();
  if (!(state as typeof DEEP_LINK_NAV_STATE | null)?.fromDeepLink) return null;

  return (
    <Alert variant='warning' className={className}>
      <Link2 className='h-4 w-4' aria-hidden='true' />
      <AlertTitle>
        <Trans>Opened from a link</Trans>
      </AlertTitle>
      <AlertDescription>
        <Trans>
          These details were filled in by a link. Check them before you confirm.
        </Trans>
      </AlertDescription>
    </Alert>
  );
}
