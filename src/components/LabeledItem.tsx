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
  if ((!content || content.trim() === '') && !children) {
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
    >
      <label
        id={labelId}
        htmlFor={contentId}
        className='text-sm font-medium text-muted-foreground block mb-1'
      >
        {label}
      </label>

      {content && isValidUrl(content) ? (
        <button
          type='button'
          id={contentId}
          onClick={() => openUrl(content)}
          onKeyDown={(e) => handleKeyDown(e, () => openUrl(content))}
          className='text-sm break-all text-blue-700 dark:text-blue-300 cursor-pointer hover:underline focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 rounded-sm text-left w-full p-0 border-0 bg-transparent'
          title={t`Open external link: ${content}`}
          aria-label={t`${label}: ${content} (opens in external application)`}
          aria-describedby={labelId}
        >
          {content}
        </button>
      ) : onClick ? (
        <button
          type='button'
          id={contentId}
          onClick={onClick}
          onKeyDown={(e) => handleKeyDown(e, onClick)}
          className='text-sm break-all text-blue-700 dark:text-blue-300 cursor-pointer hover:underline focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 rounded-sm text-left w-full p-0 border-0 bg-transparent'
          title={t`Navigate to: ${content}`}
          aria-label={t`${label}: ${content} (navigate within app)`}
          aria-describedby={labelId}
        >
          {content}
        </button>
      ) : (
        <div
          id={contentId}
          className={`text-sm break-all ${className}`}
          aria-describedby={labelId}
          role='text'
        >
          {content}
        </div>
      )}

      {children}
    </div>
  );
}
