import Container from '@/components/Container';
import Header from '@/components/Header';
import Layout from '@/components/Layout';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover';
import { Switch } from '@/components/ui/switch';
import { useErrors } from '@/hooks/useErrors';
import { Plural, Trans } from '@lingui/react/macro';
import { platform } from '@tauri-apps/plugin-os';
import { HelpCircleIcon, Trash2Icon } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { commands, PeerRecord } from '../bindings';
import PeerList from '@/components/PeerList';

export default function PeerListPage() {
  const { addError } = useErrors();

  const [peers, setPeers] = useState<PeerRecord[] | null>(null);
  const [rowSelection, setRowSelection] = useState({});
  const [isAddOpen, setAddOpen] = useState(false);
  const [ip, setIp] = useState('');
  const [ban, setBan] = useState(false);
  const [peerToDelete, setPeerToDelete] = useState<PeerRecord[] | null>(null);
  const [selectionMode, setSelectionMode] = useState(false);
  const [selectedPeers, setSelectedPeers] = useState<Set<string>>(new Set());

  const totalPeersCount = peers?.length ?? 0;
  const selectedPeersCount = selectedPeers.size;
  const peersToDeleteCount = peerToDelete?.length ?? 0;

  const isMobile = platform() === 'ios' || platform() === 'android';

  const handleBatchDelete = () => {
    const selectedPeerIds = Object.keys(rowSelection);
    const peersToDelete = peers?.filter((p) => selectedPeerIds.includes(p.ip_addr)) ?? [];
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
                      <Trans>Add Peer</Trans>
                    </Button>
                  </DialogTrigger>
                  <DialogContent className='sm:max-w-[425px]'>
                    <DialogHeader>
                      <DialogTitle>
                        <Trans>Add new peer</Trans>
                      </DialogTitle>
                      <DialogDescription>
                        <Trans>
                          Enter the IP address of the peer you want to connect
                          to.
                        </Trans>
                      </DialogDescription>
                    </DialogHeader>
                    <div className='grid gap-4 py-4'>
                      <div className='flex flex-col space-y-1.5'>
                        <Label htmlFor='ip'>
                          <Trans>IP Address</Trans>
                        </Label>
                        <Input
                          id='ip'
                          value={ip}
                          onChange={(e) => setIp(e.target.value)}
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
                          commands.addPeer({ ip }).then((result) => {
                            if (result.status === 'error') {
                              console.error(result.error);
                            }
                          });
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
            <PeerList
              peers={peers}
              onDelete={setPeerToDelete}
              selectionMode={selectionMode}
              selectedPeers={selectedPeers}
              onSelect={handleSelect}
              rowSelection={rowSelection}
              onRowSelectionChange={setRowSelection}
            />
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
