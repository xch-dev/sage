import type { DonationDetails } from 'sage-system-app-sdk';
import { inlineImageSrc } from '../utils';

export function DeveloperCard({ details }: { details: DonationDetails }) {
  const authorAvatarSrc = inlineImageSrc(details.authorAvatar);

  return (
    <div className='flex items-center gap-3 rounded-xl border bg-background/70 p-3'>
      {authorAvatarSrc ? (
        <img
          src={authorAvatarSrc}
          alt=''
          className='h-10 w-10 rounded-full border object-cover'
        />
      ) : (
        <div className='flex h-10 w-10 items-center justify-center rounded-full border bg-muted text-sm font-semibold'>
          {(details.authorName ?? details.appName).slice(0, 1).toUpperCase()}
        </div>
      )}

      <div className='min-w-0'>
        <div className='truncate text-sm font-semibold'>
          Support {details.authorName ?? details.appName}
        </div>
        <div className='truncate text-xs text-muted-foreground'>
          Developer of {details.appName}
        </div>
      </div>
    </div>
  );
}
