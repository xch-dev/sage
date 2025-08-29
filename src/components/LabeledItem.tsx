import { isValidUrl } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { openUrl } from '@tauri-apps/plugin-opener';
import { useId } from 'react';

export interface LabeledItemProps {
  label: string;
  className?: string;
  content: string | null | undefined;
  onClick?: () => void;
  children?: React.ReactNode;
}

export function LabeledItem({
  label,
  className = '',
  content,
  onClick,
  children,
}: LabeledItemProps) {
  const labelId = useId();
  const contentId = useId();

  // Don't render if both content and children are null or empty
  if (
    (!content || (typeof content === 'string' && content.trim() === '')) &&
    !children
  ) {
    return null;
  }

  const handleKeyDown = (event: React.KeyboardEvent, action: () => void) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      action();
    }
  };

  return (
    <div
      role='region'
      aria-labelledby={labelId}
      aria-label={t`${label} section`}
      className={className}
    >
      <label
        id={labelId}
        htmlFor={contentId}
        className='text-sm font-medium text-muted-foreground block mb-0.5'
      >
        {label}
      </label>

      {content && isValidUrl(content) ? (
        <span
          id={contentId}
          onClick={() => openUrl(content)}
          onKeyDown={(e) => handleKeyDown(e, () => openUrl(content))}
          className='text-sm break-words text-blue-600 cursor-pointer hover:underline focus:outline-none rounded-sm text-left p-0 border-0 bg-transparent m-0'
          title={t`Open external link: ${content}`}
          aria-label={t`${label}: ${content} (opens in external application)`}
          aria-describedby={labelId}
          role='button'
          tabIndex={0}
        >
          {content}
        </span>
      ) : onClick ? (
        <span
          id={contentId}
          onClick={onClick}
          onKeyDown={(e) => handleKeyDown(e, onClick)}
          className='text-sm break-words text-blue-600 cursor-pointer hover:underline focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 rounded-sm text-left p-0 border-0 bg-transparent m-0'
          title={t`Navigate to: ${content}`}
          aria-label={t`${label}: ${content} (navigate within app)`}
          aria-describedby={labelId}
          role='button'
          tabIndex={0}
        >
          {content}
        </span>
      ) : (
        <span
          id={contentId}
          className={`text-sm break-words ${className}`}
          aria-describedby={labelId}
        >
          {content}
        </span>
      )}

      {children}
    </div>
  );
}
