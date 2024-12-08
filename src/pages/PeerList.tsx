import Header from '@/components/Header';
import {
  ColumnDef,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from '@tanstack/react-table';
import { useCallback, useEffect, useState } from 'react';
import { commands, PeerRecord } from '../bindings';

import { Button } from '@/components/ui/button';
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

import Container from '@/components/Container';
import Layout from '@/components/Layout';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover';
import { Switch } from '@/components/ui/switch';
import { useErrors } from '@/hooks/useErrors';
import {
  BadgeCheckIcon,
  BadgeIcon,
  BanIcon,
  HelpCircleIcon,
} from 'lucide-react';

export default function PeerList() {
  const { addError } = useErrors();

  const [peers, setPeers] = useState<PeerRecord[] | null>(null);
  const [isAddOpen, setAddOpen] = useState(false);
  const [ip, setIp] = useState('');
  const [trusted, setTrusted] = useState(true);

  const [ban, setBan] = useState(false);
  const [peerToDelete, setPeerToDelete] = useState<PeerRecord | null>(null);

  const columns: ColumnDef<PeerRecord>[] = [
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
      cell: ({ row }) => {
        const peer = row.original;

        return (
          <div className='text-center'>
            <Button
              size='icon'
              variant='ghost'
              onClick={() => setPeerToDelete(peer)}
            >
              <BanIcon className='h-4 w-4' />
            </Button>
          </div>
        );
      },
    },
  ];

  const table = useReactTable({
    data: peers ?? [],
    columns,
    getCoreRowModel: getCoreRowModel(),
  });

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
      <Header title='Peer List' />

      <Container className='max-w-2xl'>
        <Card className='rounded-md border'>
          <CardHeader>
            <div className='flex justify-between items-center'>
              <CardTitle>Connected to {peers?.length ?? 0} peers</CardTitle>
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
                    <Button variant='outline' onClick={() => setAddOpen(false)}>
                      Cancel
                    </Button>
                    <Button
                      onClick={() => {
                        commands
                          .addPeer({ ip, trusted })
                          .then(() => updatePeers())
                          .catch(addError)
                          .finally(() => {
                            setIp('');
                            setAddOpen(false);
                          });
                      }}
                      autoFocus
                    >
                      Connect
                    </Button>
                  </DialogFooter>
                </DialogContent>
              </Dialog>
            </div>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                {table.getHeaderGroups().map((headerGroup) => (
                  <TableRow key={headerGroup.id}>
                    {headerGroup.headers.map((header) => {
                      return (
                        <TableHead key={header.id} className='px-4'>
                          {header.isPlaceholder
                            ? null
                            : flexRender(
                                header.column.columnDef.header,
                                header.getContext(),
                              )}
                        </TableHead>
                      );
                    })}
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
          </CardContent>
        </Card>

        <Dialog
          open={!!peerToDelete}
          onOpenChange={(open) => !open && setPeerToDelete(null)}
        >
          <DialogContent>
            <DialogTitle>Are you sure you want to remove the peer?</DialogTitle>

            <DialogDescription>
              This will remove the peer from your connections. If you are
              currently syncing against this peer, a new one will be used to
              replace it.
            </DialogDescription>

            <div className='flex items-center space-x-2'>
              <Switch
                id='ban'
                checked={ban}
                onCheckedChange={(checked) => setBan(checked)}
              />
              <Label htmlFor='ban'>Ban peer temporarily</Label>
              <Popover>
                <PopoverTrigger>
                  <HelpCircleIcon className='h-4 w-4 text-muted-foreground' />
                </PopoverTrigger>
                <PopoverContent className='text-sm'>
                  Will temporarily prevent the peer from being connected to.
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
                  peerToDelete &&
                    commands
                      .removePeer({ ip: peerToDelete.ip_addr, ban })
                      .then(() => {
                        setPeerToDelete(null);
                        updatePeers();
                      })
                      .catch(addError)
                      .finally(() => setPeerToDelete(null));
                }}
                autoFocus
              >
                Remove Peer
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </Container>
    </Layout>
  );
}
