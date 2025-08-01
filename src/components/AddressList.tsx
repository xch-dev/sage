import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { ColumnDef, Row } from '@tanstack/react-table';
import { useMemo } from 'react';
import { toast } from 'react-toastify';
import { DerivationRecord } from '../bindings';
import { CopyButton } from './CopyButton';
import { FormattedAddress } from './FormattedAddress';
import { SimplePagination } from './SimplePagination';
import { DataTable } from './ui/data-table';

// Extract column header and cell components
const IndexHeader = () => <Trans>Index</Trans>;

const IndexCell = ({ row }: { row: Row<DerivationRecord> }) =>
  row.original.index;

const AddressHeader = () => <Trans>Address</Trans>;

const AddressCell = ({ row }: { row: Row<DerivationRecord> }) => {
  const address = row.getValue('address') as string;
  return (
    <div className='flex w-full items-center'>
      <div className='flex-grow overflow-hidden pr-2'>
        <FormattedAddress address={address} />
      </div>
      <div className='flex-shrink-0'>
        <CopyButton
          value={address}
          className='h-8'
          onCopy={() => {
            toast.success(t`Address copied to clipboard`);
          }}
        />
      </div>
    </div>
  );
};

const PublicKeyHeader = () => <Trans>Public Key</Trans>;

const PublicKeyCell = ({ row }: { row: Row<DerivationRecord> }) => {
  const publicKey = row.getValue('public_key') as string;
  return (
    <div className='flex w-full items-center'>
      <div className='flex-grow overflow-hidden pr-2 font-mono text-xs truncate'>
        {publicKey}
      </div>
      <div className='flex-shrink-0'>
        <CopyButton
          value={publicKey}
          className='h-8'
          onCopy={() => {
            toast.success(t`Public key copied to clipboard`);
          }}
        />
      </div>
    </div>
  );
};

export interface AddressListProps {
  derivations: DerivationRecord[];
  currentPage: number;
  totalPages: number;
  setCurrentPage: (page: number) => void;
  totalDerivations: number;
}

export default function AddressList(props: AddressListProps) {
  const {
    derivations,
    currentPage,
    totalPages,
    setCurrentPage,
    totalDerivations,
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
    ],
    [],
  );

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
            {t`Showing ${derivations.length} of ${totalDerivations} addresses`}
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
