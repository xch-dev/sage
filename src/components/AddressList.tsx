import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { ColumnDef, Row } from '@tanstack/react-table';
import { useMemo } from 'react';
import { SignatureIcon } from 'lucide-react';
import { DerivationRecord } from '../bindings';
import { AddressItem } from './AddressItem';
import { SimplePagination } from './SimplePagination';
import { Button } from './ui/button';
import { DataTable } from './ui/data-table';

// Extract column header and cell components
const IndexHeader = () => <Trans>Index</Trans>;

const IndexCell = ({ row }: { row: Row<DerivationRecord> }) =>
  row.original.index;

const AddressHeader = () => <Trans>Address</Trans>;

const AddressCell = ({ row }: { row: Row<DerivationRecord> }) => {
  const address = row.getValue('address') as string;
  return (
    <AddressItem
      address={address}
      label={t`Address`}
      hideLabel={true}
      inputClassName='focus-visible:ring-0 focus-visible:ring-offset-0'
      truncateMiddle={true}
    />
  );
};

const PublicKeyHeader = () => <Trans>Public Key</Trans>;

const PublicKeyCell = ({ row }: { row: Row<DerivationRecord> }) => {
  const publicKey = row.getValue('public_key') as string;
  return (
    <AddressItem
      address={publicKey}
      label={t`Public key`}
      hideLabel={true}
      inputClassName='focus-visible:ring-0 focus-visible:ring-offset-0'
    />
  );
};

const ActionsHeader = () => <span className='sr-only'>{t`Actions`}</span>;

const signMessageColumn = (
  onSignMessage: (address: string) => void,
): ColumnDef<DerivationRecord> => ({
  id: 'actions',
  size: 40,
  header: ActionsHeader,
  cell: ({ row }) => (
    <Button
      variant='ghost'
      size='icon'
      onClick={() => onSignMessage(row.original.address)}
      aria-label={t`Sign message with this address`}
      title={t`Sign message`}
    >
      <SignatureIcon className='h-4 w-4' aria-hidden='true' />
    </Button>
  ),
});

export interface AddressListProps {
  derivations: DerivationRecord[];
  currentPage: number;
  totalPages: number;
  setCurrentPage: (page: number) => void;
  totalDerivations: number;
  onSignMessage?: (address: string) => void;
}

export default function AddressList(props: AddressListProps) {
  const {
    derivations,
    currentPage,
    totalPages,
    setCurrentPage,
    totalDerivations,
    onSignMessage,
  } = props;

  const columns: ColumnDef<DerivationRecord>[] = useMemo(
    () => [
      {
        id: 'index',
        accessorFn: (row) => row.index,
        size: 40,
        header: IndexHeader,
        cell: IndexCell,
      },
      {
        id: 'address',
        accessorFn: (row) => row.address,
        header: AddressHeader,
        cell: AddressCell,
      },
      {
        id: 'public_key',
        accessorFn: (row) => row.public_key,
        header: PublicKeyHeader,
        cell: PublicKeyCell,
      },
      ...(onSignMessage ? [signMessageColumn(onSignMessage)] : []),
    ],
    [onSignMessage],
  );

  const shownDerivations = derivations.length;

  return (
    <div className='flex flex-col'>
      <div className='flex-shrink overflow-auto h-[350px]'>
        <DataTable
          columns={columns}
          data={derivations}
          getRowId={(row) => row.index.toString()}
          showTotalRows={false}
        />
      </div>
      <div className='flex-shrink-0 py-4'>
        <div className='flex items-center justify-between'>
          <div className='text-sm text-muted-foreground'>
            {t`Showing ${shownDerivations} of ${totalDerivations} addresses`}
          </div>
          <SimplePagination
            currentPage={currentPage}
            pageCount={totalPages}
            setCurrentPage={setCurrentPage}
            size='sm'
            align='end'
          />
        </div>
      </div>
    </div>
  );
}
