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
  info: 'bg-blue-50 dark:bg-blue-950 border-blue-200 dark:border-blue-800 text-blue-800 dark:text-blue-300',
  warning:
    'bg-amber-50 dark:bg-amber-950 border-amber-200 dark:border-amber-800 text-amber-800 dark:text-amber-300',
  danger:
    'bg-red-50 dark:bg-red-950 border-red-200 dark:border-red-800 text-red-800 dark:text-red-300',
  success:
    'bg-emerald-50 dark:bg-emerald-950 border-emerald-200 dark:border-emerald-800 text-emerald-800 dark:text-emerald-300',
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
        <Icon className='h-3 w-3 mr-1' />
        {title}
      </div>
      <div>{children}</div>
    </div>
  );
}
