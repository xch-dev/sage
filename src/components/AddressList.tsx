import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { ColumnDef, SortingState } from '@tanstack/react-table';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import { useMemo, useState } from 'react';
import { FormattedAddress } from './FormattedAddress';
import { Button } from './ui/button';
import { toast } from 'react-toastify';
import { DataTable } from './ui/data-table';
import { CopyButton } from './CopyButton';
import { CopyBox } from './CopyBox';
import { formatAddress } from '@/lib/utils';

export interface AddressListProps {
  addresses: string[];
}

interface AddressRow {
  id: number;
  address: string;
}

export default function AddressList(props: AddressListProps) {
  const [sorting, setSorting] = useState<SortingState>([
    { id: 'id', desc: false },
  ]);
  const [currentPage, setCurrentPage] = useState(0);
  const pageSize = 100;

  const rows = useMemo(() => {
    return props.addresses.map((address, i) => ({
      id: i,
      address,
    }));
  }, [props.addresses]);

  // Calculate pagination
  const pageCount = Math.ceil(rows.length / pageSize);
  const paginatedRows = rows.slice(
    currentPage * pageSize,
    (currentPage + 1) * pageSize,
  );

  const columns: ColumnDef<AddressRow>[] = [
    {
      id: 'id',
      accessorFn: (row) => row.id,
      meta: {
        className: 'w-14 text-center',
      },
      header: () => (
        <div className='text-center'>
          <Trans>Index</Trans>
        </div>
      ),
      cell: (info) => <div>{info.row.original.id}</div>,
    },
    {
      id: 'address',
      accessorFn: (row) => row.address,
      meta: {
        className: 'w-full',
      },
      header: () => <Trans>Address</Trans>,
      cell: (info) => {
        const address = info.getValue() as string;
        return (
          <div className='flex w-full'>
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
  ];

  return (
    <div className='flex flex-col'>
      <div className='flex-shrink overflow-auto h-[350px]'>
        <DataTable
          columns={columns}
          data={paginatedRows}
          state={{
            sorting,
          }}
          onSortingChange={setSorting}
          getRowId={(row) => row.id.toString()}
        />
      </div>
      <div className='flex-shrink-0 flex items-center justify-end space-x-2 py-4'>
        <Button
          variant='outline'
          size='sm'
          onClick={() => setCurrentPage(Math.max(0, currentPage - 1))}
          disabled={currentPage === 0}
          aria-label={t`Previous page`}
        >
          <ChevronLeft className='h-4 w-4' aria-hidden='true' />
        </Button>
        <Button
          variant='outline'
          size='sm'
          onClick={() =>
            setCurrentPage(Math.min(pageCount - 1, currentPage + 1))
          }
          disabled={currentPage >= pageCount - 1}
          aria-label={t`Next page`}
        >
          <ChevronRight className='h-4 w-4' aria-hidden='true' />
        </Button>
      </div>
    </div>
  );
}
