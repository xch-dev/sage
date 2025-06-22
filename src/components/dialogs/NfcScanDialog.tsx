import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Trans } from '@lingui/react/macro';

interface NfcScanDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function NfcScanDialog({ open, onOpenChange }: NfcScanDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Scan an NFC tag</DialogTitle>
          <DialogDescription>
            <Trans>Place the NFC tag near your device to scan it.</Trans>
          </DialogDescription>
        </DialogHeader>
      </DialogContent>
    </Dialog>
  );
}
