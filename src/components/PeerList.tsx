import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { useLongPress } from '@/hooks/useLongPress';
import { Trans } from '@lingui/react/macro';
import { animated, useSpring } from '@react-spring/web';
import {
  ColumnDef,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from '@tanstack/react-table';
import { platform } from '@tauri-apps/plugin-os';
import { useDrag } from '@use-gesture/react';
import { BanIcon, Trash2Icon } from 'lucide-react';
import { PeerRecord } from '../bindings';
import { t } from '@lingui/core/macro';

interface PeerListProps {
  peers: PeerRecord[] | null;
  onDelete: (peers: PeerRecord[]) => void;
  selectionMode: boolean;
  selectedPeers: Set<string>;
  onSelect: (peer: PeerRecord, forceModeOn?: boolean) => void;
  rowSelection: Record<string, boolean>;
  onRowSelectionChange: (value: Record<string, boolean>) => void;
}

const MobileRow = ({
  peer,
  onDelete,
  selected,
  onSelect,
  selectionMode,
}: {
  peer: PeerRecord;
  onDelete: () => void;
  selected: boolean;
  onSelect: (peer: PeerRecord, forceModeOn?: boolean) => void;
  selectionMode: boolean;
}) => {
  const [{ x }, api] = useSpring(() => ({
    x: 0,
    config: { tension: 400, friction: 30 },
  }));

  const handleDelete = () => {
    api.start({
      x: 0,
      onRest: () => {
        onDelete();
      },
    });
  };

  const bind = useDrag(
    ({ down, movement: [mx], cancel }) => {
      if (selectionMode || mx > 0) {
        cancel();
        return;
      }

      const curX = mx * 0.8;

      if (down) {
        api.start({ x: curX, immediate: true });
      } else if (curX < -70) {
        api.start({ x: -100, onRest: handleDelete });
      } else {
        api.start({ x: 0 });
      }
    },
    {
      axis: 'x',
      bounds: { left: -100, right: 0 },
      from: () => [x.get(), 0],
      filterTaps: true,
    },
  );

  const longPressHandlers = useLongPress(
    () => onSelect(peer, true),
    () => selectionMode && onSelect(peer),
  );

  return (
    <div className='relative overflow-hidden border-b last:border-b-0'>
      <div className='absolute inset-y-0 right-0 w-20 bg-red-500 flex items-center justify-center'>
        <Trash2Icon className='h-5 w-5 text-white' />
      </div>

      <animated.div
        {...bind()}
        {...longPressHandlers}
        style={{ x }}
        className='relative bg-background p-4 touch-pan-y select-none'
      >
        <div className='flex items-center space-x-3'>
          {selectionMode && (
            <Checkbox
              checked={selected}
              onCheckedChange={() => onSelect(peer)}
              className='mr-2'
            />
          )}
          <span className='font-medium flex-1'>{peer.ip_addr}</span>
        </div>

        <div className='mt-3 grid grid-cols-2 gap-2 text-sm text-muted-foreground'>
          <div className='flex items-center space-x-2'>
            <span className='text-gray-500'>
              <Trans>Height:</Trans>
            </span>
            <span>{peer.peak_height.toLocaleString()}</span>
          </div>
          <div className='flex items-center justify-end space-x-2'>
            <span className='text-gray-500'>
              <Trans>Port:</Trans>
            </span>
            <span>{peer.port}</span>
          </div>
        </div>
      </animated.div>

      <Button 
        onClick={handleDelete}
        aria-label={t`Delete peer ${peer.ip_addr}`}
        className="sr-only"
      >
        <Trans>Delete peer</Trans>
      </Button>
    </div>
  );
};

export default function PeerList({
  peers,
  onDelete,
  selectionMode,
  selectedPeers,
  onSelect,
  rowSelection,
  onRowSelectionChange,
}: PeerListProps) {
  const isMobile = platform() === 'ios' || platform() === 'android';

  const columns: ColumnDef<PeerRecord>[] = [
    {
      id: 'select',
      header: ({ table }) => (
        <Checkbox
          className='mx-2'
          checked={
            table.getIsAllPageRowsSelected() ||
            (table.getIsSomePageRowsSelected() && 'indeterminate')
          }
          onCheckedChange={(value) => table.toggleAllPageRowsSelected(!!value)}
          aria-label={t`Select all peers`}
        />
      ),
      cell: ({ row }) => (
        <Checkbox
          className='mx-2'
          checked={row.getIsSelected()}
          onCheckedChange={(value) => row.toggleSelected(!!value)}
          aria-label={t`Select peer ${row.original.ip_addr}`}
        />
      ),
    },
    {
      accessorKey: 'ip_addr',
      header: () => <Trans>IP Address</Trans>,
    },
    {
      accessorKey: 'port',
      header: () => <Trans>Port</Trans>,
    },
    {
      accessorKey: 'peak_height',
      header: () => <Trans>Height</Trans>,
    },
    {
      id: 'actions',
      header: () => (
        <div className='text-center'>
          <Trans>Actions</Trans>
        </div>
      ),
      cell: ({ row }) => (
        <div className='text-center'>
          <Button
            size='icon'
            variant='ghost'
            onClick={() => onDelete([row.original])}
          >
            <BanIcon className='h-4 w-4' />
          </Button>
        </div>
      ),
    },
  ];

  const table = useReactTable({
    data: peers ?? [],
    columns,
    getCoreRowModel: getCoreRowModel(),
    enableRowSelection: true,
    state: {
      rowSelection,
    },
    onRowSelectionChange: (updaterOrValue) => {
      onRowSelectionChange(
        typeof updaterOrValue === 'function' 
          ? updaterOrValue(rowSelection)
          : updaterOrValue
      );
    },
  });

  return isMobile ? (
    <div>
      {peers?.map((peer) => (
        <MobileRow
          key={peer.ip_addr}
          peer={peer}
          onDelete={() => onDelete([peer])}
          selected={selectedPeers.has(peer.ip_addr)}
          onSelect={onSelect}
          selectionMode={selectionMode}
        />
      ))}
    </div>
  ) : (
    <Table role="grid" aria-label={t`Peer List`}>
      <TableHeader>
        {table.getHeaderGroups().map((headerGroup) => (
          <TableRow key={headerGroup.id}>
            {headerGroup.headers.map((header) => (
              <TableHead key={header.id} className='px-4'>
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
            <TableRow
              key={row.id}
              data-state={row.getIsSelected() && 'selected'}
            >
              {row.getVisibleCells().map((cell) => (
                <TableCell key={cell.id} className='px-4'>
                  {flexRender(cell.column.columnDef.cell, cell.getContext())}
                </TableCell>
              ))}
            </TableRow>
          ))
        ) : (
          <TableRow>
            <TableCell colSpan={columns.length} className='h-24 text-center'>
              <Trans>No results.</Trans>
            </TableCell>
          </TableRow>
        )}
      </TableBody>
    </Table>
  );
} 