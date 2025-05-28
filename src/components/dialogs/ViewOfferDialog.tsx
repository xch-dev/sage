import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Textarea } from '@/components/ui/textarea';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';

interface ViewOfferDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  offerString: string;
  setOfferString: (value: string) => void;
  onSubmit: (event: React.FormEvent<HTMLFormElement>) => void;
}

export function ViewOfferDialog({
  open,
  onOpenChange,
  offerString,
  setOfferString,
  onSubmit,
}: ViewOfferDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            <Trans>Enter Offer String or Dexie URL</Trans>
          </DialogTitle>
        </DialogHeader>
        <form onSubmit={onSubmit} className='flex flex-col gap-4'>
          <Textarea
            placeholder={t`Paste your offer string or Dexie URL here...`}
            value={offerString}
            onChange={(e) => setOfferString(e.target.value)}
            className='min-h-[200px] font-mono text-xs'
          />
          <Button type='submit'>
            <Trans>View Offer</Trans>
          </Button>
        </form>
      </DialogContent>
    </Dialog>
  );
}
