import { useCallback, useEffect, useRef, useState } from 'react';
import { commands, PeerRecord } from '../bindings';
import { platform } from '@tauri-apps/plugin-os';
import {
  ColumnDef,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from '@tanstack/react-table';
import Header from '@/components/Header';
import Container from '@/components/Container';
import Layout from '@/components/Layout';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Checkbox } from '@/components/ui/checkbox';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover';
import { Switch } from '@/components/ui/switch';
import {
  BadgeCheckIcon,
  BadgeIcon,
  BanIcon,
  HelpCircleIcon,
  Trash2Icon,
} from 'lucide-react';
import { animated, useSpring } from '@react-spring/web';
import { useDrag } from '@use-gesture/react';
import { useLongPress } from '@/hooks/useLongPress';

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
          {peer.trusted ? (
            <BadgeCheckIcon className='h-5 w-5 text-blue-500' />
          ) : (
            <BadgeIcon className='h-5 w-5 text-gray-400' />
          )}
        </div>

        <div className='mt-3 grid grid-cols-2 gap-2 text-sm text-muted-foreground'>
          <div className='flex items-center space-x-2'>
            <span className='text-gray-500'>Height:</span>
            <span>{peer.peak_height.toLocaleString()}</span>
          </div>
          <div className='flex items-center justify-end space-x-2'>
            <span className='text-gray-500'>Port:</span>
            <span>{peer.port}</span>
          </div>
        </div>
      </animated.div>
    </div>
  );
};

export default function PeerList() {
  const [peers, setPeers] = useState<PeerRecord[] | null>(null);
  const [rowSelection, setRowSelection] = useState({});
  const [isAddOpen, setAddOpen] = useState(false);
  const [ip, setIp] = useState('');
  const [trusted, setTrusted] = useState(true);
  const [ban, setBan] = useState(false);
  const [peerToDelete, setPeerToDelete] = useState<PeerRecord[] | null>(null);
  const [selectionMode, setSelectionMode] = useState(false);
  const [selectedPeers, setSelectedPeers] = useState(new Set());

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
          aria-label='Select all'
        />
      ),
      cell: ({ row }) => (
        <Checkbox
          className='mx-2'
          checked={row.getIsSelected()}
          onCheckedChange={(value) => row.toggleSelected(!!value)}
          aria-label='Select row'
        />
      ),
    },
    {
      accessorKey: 'ip_addr',
      header: 'IP Address',
    },
    {
      accessorKey: 'port',
      header: 'Port',
    },
    {
      accessorKey: 'peak_height',
      header: 'Peak Height',
    },
    {
      accessorKey: 'trusted',
      header: () => <div className='text-center'>Trusted</div>,
      cell: ({ row }) => (
        <div className='flex items-center justify-center'>
          {row.original.trusted ? (
            <BadgeCheckIcon className='h-4 w-4' />
          ) : (
            <BadgeIcon className='h-4 w-4 text-muted-foreground' />
          )}
        </div>
      ),
    },
    {
      id: 'actions',
      header: () => <div className='text-center'>Actions</div>,
      cell: ({ row }) => (
        <div className='text-center'>
          <Button
            size='icon'
            variant='ghost'
            onClick={() => setPeerToDelete([row.original])}
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
    onRowSelectionChange: setRowSelection,
  });

  const handleBatchDelete = () => {
    const selectedRows = table.getSelectedRowModel().rows;
    const peersToDelete = selectedRows.map((row) => row.original);
    if (peersToDelete.length > 0) {
      setPeerToDelete(peersToDelete);
    }
  };

  const handleSelect = (peer: PeerRecord, forceModeOn = false) => {
    if (forceModeOn && !selectionMode) {
      setSelectionMode(true);
      setSelectedPeers(new Set([peer.ip_addr]));
      return;
    }

    setSelectedPeers((prev) => {
      const newSet = new Set(prev);
      if (newSet.has(peer.ip_addr)) {
        newSet.delete(peer.ip_addr);
        if (newSet.size === 0) {
          setSelectionMode(false);
        }
      } else {
        newSet.add(peer.ip_addr);
      }
      return newSet;
    });
  };

  const updatePeers = () => {
    commands.getPeers({}).then((res) => {
      if (res.status === 'ok') {
        setPeers(res.data.peers);
      }
    });
  };

  useEffect(() => {
    updatePeers();
    const interval = setInterval(updatePeers, 1000);
    return () => clearInterval(interval);
  }, []);

  return (
    <Layout>
      <Header title='Peer List' />
      <Container className='max-w-2xl'>
        <Card className='rounded-md border'>
          <CardHeader>
            <div className='flex justify-between items-center'>
              <CardTitle className='flex-1'>
                {selectionMode
                  ? `Selected ${selectedPeers.size} of ${peers?.length ?? 0} peers`
                  : `Connected to ${peers?.length ?? 0} peers`}
              </CardTitle>
              {selectionMode ? (
                <div className='flex space-x-2'>
                  <Button
                    variant='outline'
                    onClick={() => {
                      setSelectionMode(false);
                      setSelectedPeers(new Set());
                    }}
                  >
                    Cancel
                  </Button>
                  <Button
                    variant='destructive'
                    onClick={() => {
                      const peersToDelete =
                        peers?.filter((peer) =>
                          selectedPeers.has(peer.ip_addr),
                        ) ?? [];
                      if (peersToDelete.length > 0) {
                        setPeerToDelete(peersToDelete);
                      }
                    }}
                    disabled={selectedPeers.size === 0}
                  >
                    Delete ({selectedPeers.size})
                  </Button>
                </div>
              ) : (
                <Dialog open={isAddOpen} onOpenChange={setAddOpen}>
                  <DialogTrigger asChild>
                    <Button variant='outline'>Add Peer</Button>
                  </DialogTrigger>
                  <DialogContent className='sm:max-w-[425px]'>
                    <DialogHeader>
                      <DialogTitle>Add new peer</DialogTitle>
                      <DialogDescription>
                        Enter the IP address of the peer you want to connect to.
                      </DialogDescription>
                    </DialogHeader>
                    <div className='grid gap-4 py-4'>
                      <div className='flex flex-col space-y-1.5'>
                        <Label htmlFor='ip'>IP Address</Label>
                        <Input
                          id='ip'
                          value={ip}
                          onChange={(e) => setIp(e.target.value)}
                        />
                      </div>
                      <div className='flex items-center space-x-2'>
                        <Switch
                          id='trusted'
                          checked={trusted}
                          onCheckedChange={(checked) => setTrusted(checked)}
                        />
                        <Label htmlFor='trusted'>Trusted peer</Label>
                        <Popover>
                          <PopoverTrigger>
                            <HelpCircleIcon className='h-4 w-4 text-muted-foreground' />
                          </PopoverTrigger>
                          <PopoverContent className='text-sm'>
                            Prevents the peer from being banned.
                          </PopoverContent>
                        </Popover>
                      </div>
                    </div>
                    <DialogFooter>
                      <Button
                        variant='outline'
                        onClick={() => setAddOpen(false)}
                      >
                        Cancel
                      </Button>
                      <Button
                        onClick={() => {
                          setAddOpen(false);
                          commands.addPeer({ ip, trusted }).then((result) => {
                            if (result.status === 'error') {
                              console.error(result.error);
                            }
                          });
                          setIp('');
                        }}
                        autoFocus
                      >
                        Connect
                      </Button>
                    </DialogFooter>
                  </DialogContent>
                  {!isMobile && (
                    <Button
                      className='ml-2'
                      variant='outline'
                      onClick={handleBatchDelete}
                      disabled={Object.keys(rowSelection).length === 0}
                    >
                      <Trash2Icon className='h-5 w-5' />
                    </Button>
                  )}
                </Dialog>
              )}
            </div>
          </CardHeader>
          <CardContent>
            {isMobile ? (
              <div>
                {peers?.map((peer) => (
                  <MobileRow
                    key={peer.ip_addr}
                    peer={peer}
                    onDelete={() => setPeerToDelete([peer])}
                    selected={selectedPeers.has(peer.ip_addr)}
                    onSelect={handleSelect}
                    selectionMode={selectionMode}
                  />
                ))}
              </div>
            ) : (
              <Table>
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
            )}
          </CardContent>
        </Card>
        <Dialog
          open={!!peerToDelete}
          onOpenChange={(open) => !open && setPeerToDelete(null)}
        >
          <DialogContent>
            <DialogTitle>
              {peerToDelete?.length === 1
                ? 'Are you sure you want to remove the peer?'
                : `Are you sure you want to remove ${peerToDelete?.length} peers?`}
            </DialogTitle>
            <DialogDescription>
              This will remove the peer{peerToDelete?.length === 1 ? '' : 's'}{' '}
              from your connections. If you are currently syncing against{' '}
              {peerToDelete?.length === 1 ? 'this peer' : 'these peers'},{' '}
              {peerToDelete?.length === 1 ? 'a new one' : 'new ones'} will be
              used to replace {peerToDelete?.length === 1 ? 'it' : 'them'}.
            </DialogDescription>
            <div className='flex items-center space-x-2'>
              <Switch id='ban' checked={ban} onCheckedChange={setBan} />
              <Label htmlFor='ban'>
                Ban peer{peerToDelete?.length === 1 ? '' : 's'} temporarily
              </Label>
              <Popover>
                <PopoverTrigger>
                  <HelpCircleIcon className='h-4 w-4 text-muted-foreground' />
                </PopoverTrigger>
                <PopoverContent className='text-sm'>
                  Will temporarily prevent the peer
                  {peerToDelete?.length === 1 ? '' : 's'} from being connected
                  to.
                </PopoverContent>
              </Popover>
            </div>
            <DialogFooter>
              <Button
                type='button'
                variant='secondary'
                onClick={() => setPeerToDelete(null)}
              >
                Cancel
              </Button>
              <Button
                onClick={() => {
                  if (peerToDelete) {
                    setSelectionMode(false);
                    Promise.all(
                      peerToDelete.map((peer) =>
                        commands.removePeer({ ip: peer.ip_addr, ban }),
                      ),
                    ).then(() => {
                      setPeerToDelete(null);
                      updatePeers();
                    });
                  }
                }}
                autoFocus
              >
                Remove {peerToDelete?.length === 1 ? 'Peer' : 'Peers'}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </Container>
    </Layout>
  );
}
