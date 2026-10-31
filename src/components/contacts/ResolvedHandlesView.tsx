import { commands, ContactRecord, events, HandleRecord } from '@/bindings';
import { useWallet } from '@/contexts/WalletContext';
import { useErrors } from '@/hooks/useErrors';
import { handleState } from '@/lib/contacts';
import { toCustomError } from '@/lib/errors';
import { formatTimestamp } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { AlertTriangleIcon, BookmarkPlusIcon, UserIcon } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { CopyButton } from '../CopyButton';
import { Button } from '../ui/button';
import { ContactDialog } from './ContactDialog';
import { ContactStatusBadge } from './ContactStatusBadge';

const PAGE_SIZE = 25;

/** The most handle contacts loaded to label the handles on a page. */
const CONTACT_LIMIT = 1000;

/**
 * Every handle Sage has resolved on this network, in any wallet, from the verified mapping
 * store, whether or not it's saved as a contact. Nothing here is looked up
 * again, so each handle shows when it was last verified.
 */
export function ResolvedHandlesView() {
  const { wallet } = useWallet();
  const { addError } = useErrors();
  const [page, setPage] = useState(0);
  const [handles, setHandles] = useState<HandleRecord[]>([]);
  const [total, setTotal] = useState(0);
  const [loaded, setLoaded] = useState(false);
  const [unsupported, setUnsupported] = useState(false);
  const [contacts, setContacts] = useState<Map<string, ContactRecord>>(
    new Map(),
  );
  const [saving, setSaving] = useState<{
    name: string;
    recipient: string;
  } | null>(null);

  const refresh = useCallback(() => {
    if (!wallet) return;

    commands
      .getHandles({
        filter: 'all',
        include_unverified: true,
        refresh: false,
        offset: page * PAGE_SIZE,
        limit: PAGE_SIZE,
      })
      .then((data) => {
        setHandles(data.handles);
        setTotal(data.total);
        setUnsupported(false);
        setLoaded(true);
      })
      .catch((error) => {
        if (error?.handle_error_code === 'unsupported_network') {
          setUnsupported(true);
          setLoaded(true);
        } else {
          addError(toCustomError(error));
        }
      });

    commands
      .getContacts({
        search: null,
        kind: 'handle',
        offset: 0,
        limit: CONTACT_LIMIT,
      })
      .then((data) =>
        setContacts(
          new Map(
            data.contacts.map((contact) => [contact.handle ?? '', contact]),
          ),
        ),
      )
      .catch((error) => addError(toCustomError(error)));
  }, [wallet, page, addError]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  useEffect(() => {
    const unlisten = events.syncEvent.listen((event) => {
      if (event.payload.type === 'coin_state') refresh();
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [refresh]);

  if (unsupported) {
    return (
      <div className='text-sm text-muted-foreground py-6 text-center'>
        <Trans>XCHandles isn&apos;t available on this network.</Trans>
      </div>
    );
  }

  const pageCount = Math.max(1, Math.ceil(total / PAGE_SIZE));

  return (
    <div className='flex flex-col gap-4'>
      <p className='text-sm text-muted-foreground'>
        <Trans>
          Every handle resolved on this network, in any of your wallets, as it
          was last verified. Handles are verified again whenever you send to
          them.
        </Trans>
      </p>

      {loaded && handles.length === 0 && (
        <div className='text-sm text-muted-foreground py-6 text-center'>
          <Trans>
            No handles have been resolved yet. Enter an @handle as a recipient
            to resolve it.
          </Trans>
        </div>
      )}

      <div className='flex flex-col divide-y rounded-md border'>
        {handles.map((record) => (
          <HandleRow
            key={record.handle}
            record={record}
            contact={contacts.get(record.handle)}
            onSave={() =>
              setSaving({ name: record.handle, recipient: `@${record.handle}` })
            }
          />
        ))}
      </div>

      {total > PAGE_SIZE && (
        <div className='flex items-center justify-end gap-2 text-sm'>
          <Button
            variant='outline'
            size='sm'
            disabled={page === 0}
            onClick={() => setPage((page) => page - 1)}
          >
            <Trans>Previous</Trans>
          </Button>
          <span className='text-muted-foreground'>
            {page + 1} / {pageCount}
          </span>
          <Button
            variant='outline'
            size='sm'
            disabled={page + 1 >= pageCount}
            onClick={() => setPage((page) => page + 1)}
          >
            <Trans>Next</Trans>
          </Button>
        </div>
      )}

      <ContactDialog
        open={!!saving}
        setOpen={(open) => !open && setSaving(null)}
        initial={saving ?? undefined}
        onSaved={refresh}
      />
    </div>
  );
}

function HandleRow({
  record,
  contact,
  onSave,
}: {
  record: HandleRecord;
  contact: ContactRecord | undefined;
  onSave: () => void;
}) {
  const lastVerified = formatTimestamp(
    record.last_verified_at,
    'medium',
    'short',
  );
  const height = record.verified_height.toLocaleString();
  const name = contact?.name;
  const lastSent = record.last_sent_address;
  const changedFrom = record.changed_from;

  return (
    <div className='flex flex-col gap-2 p-3'>
      <div className='flex items-start justify-between gap-2'>
        <div className='flex flex-wrap items-center gap-2 min-w-0'>
          <span className='font-medium font-mono'>@{record.handle}</span>
          <ContactStatusBadge state={handleState(record)} />
          {name && (
            <span className='inline-flex items-center gap-1 text-xs text-muted-foreground'>
              <UserIcon className='h-3 w-3' aria-hidden='true' />
              {name}
            </span>
          )}
        </div>
        {!contact && (
          <Button variant='outline' size='sm' onClick={onSave}>
            <BookmarkPlusIcon className='h-4 w-4 mr-2' aria-hidden='true' />
            <Trans>Save as Contact</Trans>
          </Button>
        )}
      </div>

      <div className='flex items-center gap-2'>
        <div className='min-w-0 flex-1 text-xs'>
          <span className='text-muted-foreground'>
            <Trans>Last verified address</Trans>{' '}
          </span>
          <span className='font-mono break-all'>{record.address}</span>
        </div>
        <CopyButton
          value={record.address}
          className='h-7 w-7 shrink-0'
          aria-label={t`Copy address`}
        />
      </div>

      <div className='text-xs text-muted-foreground'>
        <Trans>
          Last verified {lastVerified}, at height {height}
        </Trans>
        {record.verification_reason && (
          <span> · {record.verification_reason}</span>
        )}
      </div>

      {lastSent && (
        <div className='text-xs text-muted-foreground'>
          <Trans>
            Last sent to <span className='font-mono break-all'>{lastSent}</span>
          </Trans>
        </div>
      )}

      {changedFrom && (
        <div className='flex items-start gap-1.5 text-xs text-destructive'>
          <AlertTriangleIcon
            className='h-3.5 w-3.5 mt-0.5 shrink-0'
            aria-hidden='true'
          />
          <span className='break-words'>
            <Trans>
              This handle changed. It previously resolved to{' '}
              <span className='font-mono break-all'>{changedFrom}</span>.
              You&apos;ll be asked to confirm the new address before sending to
              it.
            </Trans>
          </span>
        </div>
      )}
    </div>
  );
}
