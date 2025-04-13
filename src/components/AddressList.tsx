import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { ColumnDef } from '@tanstack/react-table';
import { toast } from 'react-toastify';
import { DerivationRecord } from '../bindings';
import { CopyButton } from './CopyButton';
import { FormattedAddress } from './FormattedAddress';
import { SimplePagination } from './SimplePagination';
import { DataTable } from './ui/data-table';

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

  const columns: ColumnDef<DerivationRecord>[] = [
    {
      id: 'index',
      accessorFn: (row) => row.index,
      size: 40,
      header: () => <Trans>Index</Trans>,
      cell: (info) => info.row.original.index,
    },
    {
      id: 'address',
      accessorFn: (row) => row.address,
      header: () => <Trans>Address</Trans>,
      cell: (info) => {
        const address = info.getValue() as string;
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
      },
    },
    {
      id: 'public_key',
      accessorFn: (row) => row.public_key,
      header: () => <Trans>Public Key</Trans>,
      cell: (info) => {
        const publicKey = info.getValue() as string;
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
      },
    },
  ];

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
