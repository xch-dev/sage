import { LatestRelease } from '@/lib/updates';
import { t } from '@lingui/core/macro';
import { openUrl } from '@tauri-apps/plugin-opener';
import { CircleArrowUp } from 'lucide-react';
import { toast } from 'react-toastify';

/** One toast per version, so a newer release replaces the notice for an older one. */
export function updateToastId(version: string) {
  return `update-available-${version}`;
}

/**
 * Tells the user a new release is out. Clicking it opens the release page. The toast stays until
 * it's dismissed, since an update notice that disappears on a timer is easy to miss. `onDismiss`
 * runs when the user closes or clicks it, but not when the app hides it.
 */
export function showUpdateToast(update: LatestRelease, onDismiss: () => void) {
  const updateVersion = update.version;
  // The toast renders outside Lingui's provider, so it can't use <Trans>.
  const message = t`Sage ${updateVersion} is available.`;
  const hint = t`Click to view the release.`;

  toast.info(
    <div className='flex flex-col'>
      <span>{message}</span>
      <span className='text-xs opacity-70'>{hint}</span>
    </div>,
    {
      toastId: updateToastId(update.version),
      icon: <CircleArrowUp className='text-primary' aria-hidden='true' />,
      autoClose: false,
      // Clicking the toast opens the release, then closes it like the close button does.
      closeOnClick: true,
      onClick: () => void openUrl(update.url),
      // react-toastify passes `true` when the user closed the toast.
      onClose: (reason) => {
        if (reason === true) onDismiss();
      },
    },
  );
}

export function hideUpdateToast(version: string) {
  toast.dismiss(updateToastId(version));
}
