import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Trans } from '@lingui/react/macro';

interface MigrationDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onCancel: () => void | Promise<void>;
  onConfirm: () => void | Promise<void>;
}

export function MigrationDialog({
  open,
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
            <Trans>
              In order to proceed with the update, the wallet will be fully
              resynced. This means any imported offer files or custom asset
              names will be removed, but you can manually add them again after
              if needed.
            </Trans>
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
