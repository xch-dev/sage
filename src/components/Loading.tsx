import { cn } from '@/lib/utils';
import { Loader2 } from 'lucide-react';

interface LoadingProps extends React.HTMLAttributes<HTMLDivElement> {
  size?: number;
  text?: string;
}

export function Loading({
  size = 24,
  text,
  className,
  ...props
}: LoadingProps) {
  return (
    <div
      className={cn(
        'flex flex-col items-center justify-center gap-2',
        className,
      )}
      {...props}
    >
      <Loader2 className='animate-spin' style={{ width: size, height: size }} />
      {text && <p className='text-sm text-muted-foreground'>{text}</p>}
    </div>
  );
}
