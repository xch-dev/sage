import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import {
  ColumnDef,
  flexRender,
  getCoreRowModel,
  getPaginationRowModel,
  useReactTable,
} from '@tanstack/react-table';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import { useMemo } from 'react';
import { Button } from './ui/button';
import { CopyButton } from './CopyButton';

export interface AddressListProps {
  addresses: string[];
}

interface AddressRow {
  id: number;
  address: string;
}

export default function AddressList(props: AddressListProps) {
  const rows = useMemo(() => {
    return props.addresses.map((address, i) => ({
      id: i,
      address,
    }));
  }, [props.addresses]);

  const columns: ColumnDef<AddressRow>[] = [
    {
      accessorKey: 'id',
      header: () => <div className='text-center'>Index</div>,
      cell: (info) => <div className='text-center'>{info.row.original.id}</div>,
    },
    {
      accessorKey: 'address',
      header: 'Address',
      cell: (info) => (
        <div className='truncate'>{info.getValue() as string}</div>
      ),
      size: 140,
    },
    {
      header: 'Actions',
      cell: (info) => (
        <div>
          <CopyButton
            value={info.row.original.address}
            className='flex-shrink-0'
          />
        </div>
      ),
    },
  ];

  const table = useReactTable({
    data: rows,
    columns,
    getCoreRowModel: getCoreRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    initialState: {
      pagination: {
        pageSize: 10,
      },
    },
  });

  return (
    <div className='flex flex-col h-full'>
      <div className='flex-shrink overflow-auto'>
        <div className='rounded-md border'>
          <Table>
            <TableHeader>
              {table.getHeaderGroups().map((headerGroup) => (
                <TableRow key={headerGroup.id}>
                  {headerGroup.headers.map((header) => (
                    <TableHead key={header.id}>
                      {header.isPlaceholder
                        ? null
                        : flexRender(
                            header.column.columnDef.header,
                            header.getContext(),
                          )}
                    </TableHead>
                  ))}
                </TableRow>
              ))}
            </TableHeader>
            <TableBody>
              {table.getRowModel().rows?.length ? (
                table.getRowModel().rows.map((row) => (
                  <TableRow key={row.id} className='-inset-x-2/40px]'>
                    {row.getVisibleCells().map((cell) => (
                      <TableCell
                        key={cell.id}
                        style={{ maxWidth: `${cell.column.getSize()}px` }}
                      >
                        {flexRender(
                          cell.column.columnDef.cell,
                          cell.getContext(),
                        )}
                      </TableCell>
                    ))}
                  </TableRow>
                ))
              ) : (
                <TableRow>
                  <TableCell
                    colSpan={columns.length}
                    className='h-24 text-center'
                  >
                    No results.
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
        </div>
      </div>
      <div className='flex-shrink-0 flex items-center justify-end space-x-2 py-4'>
        <Button
          variant='outline'
          size='sm'
          onClick={() => table.previousPage()}
          disabled={!table.getCanPreviousPage()}
        >
          <ChevronLeft className='h-4 w-4' />
        </Button>
        <Button
          variant='outline'
          size='sm'
          onClick={() => table.nextPage()}
          disabled={!table.getCanNextPage()}
        >
          <ChevronRight className='h-4 w-4' />
        </Button>
      </div>
    </div>
  );
}
