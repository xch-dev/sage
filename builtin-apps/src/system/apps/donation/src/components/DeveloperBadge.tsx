import type { DonationDetails } from 'sage-system-app-sdk';
import { inlineImageSrc } from '../utils';

export function DeveloperBadge({ details }: { details: DonationDetails }) {
  const name = details.authorName ?? details.appName;
  const avatarSrc = inlineImageSrc(details.authorAvatar);

  return (
    <span className='inline-flex max-w-full items-center gap-1.5 rounded-full border bg-background/70 py-1 pl-1 pr-2.5 align-middle'>
      {avatarSrc ? (
        <img
          src={avatarSrc}
          alt=''
          className='h-6 w-6 shrink-0 rounded-full border object-cover'
        />
      ) : (
        <span className='flex h-6 w-6 shrink-0 items-center justify-center rounded-full border bg-muted text-xs font-semibold'>
          {name.slice(0, 1).toUpperCase()}
        </span>
      )}
      <span className='truncate text-sm font-medium'>{name}</span>
    </span>
  );
}
