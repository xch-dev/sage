import { LucideIcon } from 'lucide-react';
import { ReactNode } from 'react';

export type AlertVariant = 'info' | 'warning' | 'danger' | 'success';

interface ConfirmationAlertProps {
  icon: LucideIcon;
  title: ReactNode;
  children: ReactNode;
  variant?: AlertVariant;
}

const variantStyles = {
  info: 'bg-blue-500/10 border-blue-500/30 text-blue-600',
  warning: 'bg-amber-500/10 border-amber-500/30 text-amber-600',
  danger: 'bg-red-500/10 border-red-500/30 text-red-600',
  success: 'bg-emerald-500/10 border-emerald-500/30 text-emerald-600',
};

export function ConfirmationAlert({
  icon: Icon,
  title,
  children,
  variant = 'info',
}: ConfirmationAlertProps) {
  return (
    <div className={`p-2 border rounded-md ${variantStyles[variant]}`}>
      <div className='font-medium mb-1 flex items-center'>
        <Icon className='h-3 w-3 mr-1' aria-hidden='true' />
        {title}
      </div>
      <div>{children}</div>
    </div>
  );
}
