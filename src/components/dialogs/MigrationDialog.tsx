import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { offersEnabled } from '@/lib/features';
import { Trans } from '@lingui/react/macro';

interface MigrationDialogProps {
  open: boolean;
  /** Whether the database shared by the wallets on the network failed to
   *  migrate, rather than the wallet's own. */
  shared?: boolean;
  onOpenChange: (open: boolean) => void;
  onCancel: () => void | Promise<void>;
  onConfirm: () => void | Promise<void>;
}

export function MigrationDialog({
  open,
  shared = false,
  onOpenChange,
  onCancel,
  onConfirm,
}: MigrationDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            <Trans>Database Migration Required</Trans>
          </DialogTitle>
          <DialogDescription>
            {shared ? (
              <Trans>
                In order to proceed with the update, the address book and handle
                mappings shared by the wallets on this network will be reset,
                and the wallet will be fully resynced. You can add contacts and
                custom asset names again after if needed.
              </Trans>
            ) : offersEnabled ? (
              <Trans>
                In order to proceed with the update, the wallet will be fully
                resynced. This means any imported offer files or custom asset
                names will be removed, but you can manually add them again after
                if needed.
              </Trans>
            ) : (
              <Trans>
                In order to proceed with the update, the wallet will be fully
                resynced. This means custom asset names will be removed, but you
                can manually add them again after if needed.
              </Trans>
            )}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant='outline' onClick={onCancel}>
            <Trans>Cancel</Trans>
          </Button>
          <Button variant='default' onClick={onConfirm} autoFocus>
            <Trans>OK</Trans>
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
