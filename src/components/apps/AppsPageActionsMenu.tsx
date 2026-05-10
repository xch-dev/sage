import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { commands } from '@/bindings';
import { Menu } from 'lucide-react';
import { useEffect, useState } from 'react';

interface Props {
  onOpenSandboxTests: () => void;
  onClose?: () => void;
}

export function AppsPageActionsMenu({ onOpenSandboxTests, onClose }: Props) {
  const [open, setOpen] = useState(false);

  const [autoUpdateEnabled, setAutoUpdateEnabled] = useState(false);
  const [loadingAutoUpdate, setLoadingAutoUpdate] = useState(true);

  useEffect(() => {
    let cancelled = false;

    async function load() {
      try {
        const enabled = await commands.appsGetAutoUpdateEnabled();

        if (!cancelled) {
          setAutoUpdateEnabled(enabled);
        }
      } catch (err) {
        console.error('Failed to load apps auto update setting', err);
      } finally {
        if (!cancelled) {
          setLoadingAutoUpdate(false);
        }
      }
    }

    load();

    return () => {
      cancelled = true;
    };
  }, []);

  function handleOpenChange(nextOpen: boolean) {
    setOpen(nextOpen);

    if (!nextOpen) {
      onClose?.();
    }
  }

  async function handleToggleAutoUpdate() {
    try {
      const enabled =
        await commands.appsSetAutoUpdateEnabled(!autoUpdateEnabled);

      setAutoUpdateEnabled(enabled);
    } catch (err) {
      console.error('Failed to update apps auto update setting', err);
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
        <DropdownMenuItem
          disabled={loadingAutoUpdate}
          onSelect={(event) => {
            event.preventDefault();
          }}
          onClick={handleToggleAutoUpdate}
        >
          {autoUpdateEnabled ? 'Disable auto-update' : 'Enable auto-update'}
        </DropdownMenuItem>

        <DropdownMenuSeparator />

        <DropdownMenuItem onClick={onOpenSandboxTests}>
          Sandbox tests
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
