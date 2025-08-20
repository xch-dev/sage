import { ReactNode } from 'react';

interface ConfirmationCardProps {
  icon?: ReactNode;
  title?: ReactNode;
  children: ReactNode;
  className?: string;
}

export function ConfirmationCard({
  icon,
  title,
  children,
  className = '',
}: ConfirmationCardProps) {
  return (
    <div
      className={`flex items-start gap-3 border border-border rounded-md p-3 bg-card/50 ${className}`}
    >
      {icon && (
        <div className='overflow-hidden rounded-md flex-shrink-0 w-16 h-16 border border-border flex items-center justify-center bg-muted/50'>
          {icon}
        </div>
      )}
      <div className='break-words whitespace-pre-wrap flex-1'>
        {title && <div className='font-medium mb-2'>{title}</div>}
        {children}
      </div>
    </div>
  );
}
