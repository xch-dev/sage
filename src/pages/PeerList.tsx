import Container from '@/components/Container';
import Header from '@/components/Header';
import Layout from '@/components/Layout';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';
import { Label } from '@/components/ui/label';
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover';
import { Switch } from '@/components/ui/switch';
import { useErrors } from '@/hooks/useErrors';
import { useLongPress } from '@/hooks/useLongPress';
import { t } from '@lingui/core/macro';
import { Plural, Trans } from '@lingui/react/macro';
import { animated, useSpring } from '@react-spring/web';
import {
  ColumnDef,
  getCoreRowModel,
  useReactTable,
} from '@tanstack/react-table';
import { platform } from '@tauri-apps/plugin-os';
import { useDrag } from '@use-gesture/react';
import {
  BanIcon,
  HelpCircleIcon,
  Trash2Icon,
  UserIcon,
  CableIcon,
} from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { commands, PeerRecord } from '../bindings';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { DataTable } from '@/components/ui/data-table';
import { DataTableColumnHeader } from '@/components/ui/data-table-column-header';
import { Textarea } from '@/components/ui/textarea';

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
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger>
                {peer.user_managed ? (
                  <UserIcon className='h-4 w-4 text-muted-foreground' />
                ) : (
                  <CableIcon className='h-4 w-4 text-muted-foreground' />
                )}
              </TooltipTrigger>
              <TooltipContent>
                {peer.user_managed
                  ? t`Manually added peer`
                  : t`Auto-discovered peer`}
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
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
    </div>
  );
};

export default function PeerList() {
  const { addError } = useErrors();

  const [peers, setPeers] = useState<PeerRecord[] | null>(null);
  const [rowSelection, setRowSelection] = useState({});
  const [isAddOpen, setAddOpen] = useState(false);
  const [ip, setIp] = useState('');
  const [ban, setBan] = useState(false);
  const [peerToDelete, setPeerToDelete] = useState<PeerRecord[] | null>(null);
  const [selectionMode, setSelectionMode] = useState(false);
  const [selectedPeers, setSelectedPeers] = useState(new Set());

  const totalPeersCount = peers?.length ?? 0;
  const selectedPeersCount = selectedPeers.size;
  const peersToDeleteCount = peerToDelete?.length ?? 0;

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
      size: 40,
    },
    {
      accessorKey: 'ip_addr',
      header: ({ column }) => (
        <DataTableColumnHeader column={column} title={t`IP Address`} />
      ),
      size: 150,
    },
    {
      accessorKey: 'port',
      header: ({ column }) => (
        <DataTableColumnHeader column={column} title={t`Port`} />
      ),
      size: 100,
    },
    {
      accessorKey: 'peak_height',
      header: ({ column }) => (
        <DataTableColumnHeader column={column} title={t`Height`} />
      ),
      size: 120,
    },
    {
      id: 'type',
      header: () => (
        <div className='text-center'>
          <Trans>Type</Trans>
        </div>
      ),
      size: 75,
      cell: ({ row }) => (
        <div className='text-center'>
          <TooltipProvider delayDuration={200}>
            <Tooltip>
              <TooltipTrigger asChild>
                <div className='inline-flex items-center justify-center w-8 h-8 rounded-sm hover:bg-accent'>
                  {row.original.user_managed ? (
                    <UserIcon className='h-4 w-4 text-muted-foreground' />
                  ) : (
                    <CableIcon className='h-4 w-4 text-muted-foreground' />
                  )}
                </div>
              </TooltipTrigger>
              <TooltipContent side='top' align='center' sideOffset={5}>
                {row.original.user_managed
                  ? t`Manually added peer`
                  : t`Auto-discovered peer`}
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
        </div>
      ),
    },
    {
      id: 'actions',
      header: () => (
        <div className='text-center'>
          <Trans>Actions</Trans>
        </div>
      ),
      size: 80,
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

  const updatePeers = useCallback(
    () =>
      commands
        .getPeers({})
        .then((data) => setPeers(data.peers))
        .catch(addError),
    [addError],
  );

  useEffect(() => {
    updatePeers();
    const interval = setInterval(updatePeers, 1000);

    return () => {
      clearInterval(interval);
    };
  }, [updatePeers]);

  return (
    <Layout>
      <Header title={<Trans>Peer List</Trans>} />
      <Container className='max-w-2xl'>
        <Card className='rounded-md border'>
          <CardHeader>
            <div className='flex justify-between items-center'>
              <CardTitle className='flex-1'>
                {selectionMode ? (
                  <Trans>
                    Selected {selectedPeersCount} of {totalPeersCount} peers
                  </Trans>
                ) : (
                  <Trans>Connected to {totalPeersCount} peers</Trans>
                )}
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
                    <Trans>Cancel</Trans>
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
                    <Trans>Delete {selectedPeersCount}</Trans>
                  </Button>
                </div>
              ) : (
                <Dialog open={isAddOpen} onOpenChange={setAddOpen}>
                  <DialogTrigger asChild>
                    <Button variant='outline'>
                      <Trans>Add Peers</Trans>
                    </Button>
                  </DialogTrigger>
                  <DialogContent className='sm:max-w-[425px]'>
                    <DialogHeader>
                      <DialogTitle>
                        <Trans>Add new peers</Trans>
                      </DialogTitle>
                      <DialogDescription>
                        <Trans>
                          Enter the IP addresses of the peers you want to
                          connect to.
                        </Trans>
                      </DialogDescription>
                    </DialogHeader>
                    <div className='grid gap-4 py-4'>
                      <div className='flex flex-col space-y-1.5'>
                        <Label htmlFor='ip'>
                          <Trans>IP Addresses</Trans>
                        </Label>
                        <Textarea
                          id='ip'
                          value={ip}
                          onChange={(e) => setIp(e.target.value)}
                          placeholder={t`Enter multiple IP addresses (one per line or comma-separated)`}
                          className='min-h-[100px]'
                        />
                      </div>
                    </div>
                    <DialogFooter>
                      <Button
                        variant='outline'
                        onClick={() => setAddOpen(false)}
                      >
                        <Trans>Cancel</Trans>
                      </Button>
                      <Button
                        onClick={() => {
                          setAddOpen(false);
                          // Split by newlines or commas and clean up whitespace
                          const ips = ip
                            .split(/[\n,]+/)
                            .map((ip) => ip.trim())
                            .filter(Boolean);

                          // Add each peer
                          Promise.all(
                            ips.map((ip) =>
                              commands.addPeer({ ip }).then((result) => {
                                if (result.status === 'error') {
                                  console.error(result.error);
                                }
                              }),
                            ),
                          );
                          setIp('');
                        }}
                        autoFocus
                      >
                        <Trans>Connect</Trans>
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
              <DataTable
                columns={columns}
                data={peers ?? []}
                state={{
                  rowSelection,
                }}
                onRowSelectionChange={setRowSelection}
                showTotalRows={false}
              />
            )}
          </CardContent>
        </Card>
        <Dialog
          open={!!peerToDelete}
          onOpenChange={(open) => !open && setPeerToDelete(null)}
        >
          <DialogContent>
            <DialogTitle>
              {peerToDelete?.length === 1 ? (
                <Trans>Are you sure you want to remove the peer?</Trans>
              ) : (
                <Trans>
                  Are you sure you want to remove {peersToDeleteCount} peers?
                </Trans>
              )}
            </DialogTitle>
            <DialogDescription>
              <Plural
                value={peersToDeleteCount}
                one={`This will remove the peer from your connection. If you are currently syncing against this peer, a new one will be used to replace it.`}
                other={`This will remove # peers from your connection. If you are currently syncing against these peers, new ones will be used to replace them.`}
              />
            </DialogDescription>
            <div className='flex items-center space-x-2'>
              <Switch id='ban' checked={ban} onCheckedChange={setBan} />
              <Label htmlFor='ban'>
                <Plural
                  value={peersToDeleteCount}
                  one={'Ban peer temporarily'}
                  other={'Ban peers temporarily'}
                />
              </Label>
              <Popover>
                <PopoverTrigger>
                  <HelpCircleIcon className='h-4 w-4 text-muted-foreground' />
                </PopoverTrigger>
                <PopoverContent className='text-sm'>
                  <Plural
                    value={peersToDeleteCount}
                    one={
                      'Will temporarily prevent the peer from being connected to.'
                    }
                    other={
                      'Will temporarily prevent the peers from being connected to.'
                    }
                  />
                </PopoverContent>
              </Popover>
            </div>
            <DialogFooter>
              <Button
                type='button'
                variant='secondary'
                onClick={() => setPeerToDelete(null)}
              >
                <Trans>Cancel</Trans>
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
                      setRowSelection({});
                      updatePeers();
                    });
                  }
                }}
                autoFocus
              >
                <Plural
                  value={peersToDeleteCount}
                  one={'Remove Peer'}
                  other={'Remove Peers'}
                />
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </Container>
    </Layout>
  );
}
