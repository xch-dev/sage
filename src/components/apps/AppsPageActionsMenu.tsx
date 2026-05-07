import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Menu } from 'lucide-react';
import { useState } from 'react';

interface Props {
  onOpenSandboxTests: () => void;
  onClose?: () => void;
}

export function AppsPageActionsMenu({
  onOpenSandboxTests,
  onClose,
}: Props) {
  const [open, setOpen] = useState(false);

  function handleOpenChange(nextOpen: boolean) {
    setOpen(nextOpen);
    if (!nextOpen) {
      onClose?.();
    }
  }

  return (
    <DropdownMenu open={open} onOpenChange={handleOpenChange}>
      <DropdownMenuTrigger asChild>
        <Button variant='outline' size='icon' aria-label='Open apps actions'>
          <Menu className='h-4 w-4' />
        </Button>
      </DropdownMenuTrigger>

      <DropdownMenuContent align='end' className='w-56'>
        <DropdownMenuSeparator />
        <DropdownMenuItem
          onClick={onOpenSandboxTests}
        >
          Sandbox tests
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
