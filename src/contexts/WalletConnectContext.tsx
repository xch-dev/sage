import { commands, OfferSummary, TransactionSummary } from '@/bindings';
import { AdvancedSummary } from '@/components/ConfirmationDialog';
import { OfferCard } from '@/components/OfferCard';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { useErrors } from '@/hooks/useErrors';
import useInitialization from '@/hooks/useInitialization';
import { useWallet } from '@/hooks/useWallet';
import { toDecimal } from '@/lib/utils';
import { useWalletState } from '@/state';
import {
  Params,
  WalletConnectCommand,
  walletConnectCommands,
} from '@/walletconnect/commands';
import { handleCommand } from '@/walletconnect/handler';
import { getCurrentWindow, UserAttentionType } from '@tauri-apps/api/window';
import SignClient from '@walletconnect/sign-client';
import { SessionTypes, SignClientTypes } from '@walletconnect/types';
import {
  createContext,
  ReactNode,
  useCallback,
  useEffect,
  useState,
} from 'react';

export interface WalletConnectContextType {
  sessions: any[];
  pair: (uri: string) => Promise<void>;
  disconnect: (topic: string) => Promise<void>;
}

export const WalletConnectContext = createContext<
  WalletConnectContextType | undefined
>(undefined);

type SessionRequest = SignClientTypes.EventArguments['session_request'];

export function WalletConnectProvider({ children }: { children: ReactNode }) {
  const initialized = useInitialization();
  const wallet = useWallet(initialized);
  const { addError } = useErrors();

  const [signClient, setSignClient] = useState<Awaited<
    ReturnType<typeof SignClient.init>
  > | null>(null);
  const [sessions, setSessions] = useState<SessionTypes.Struct[]>([]);
  const [pendingRequests, setPendingRequests] = useState<SessionRequest[]>([]);

  console.log('provider');

  console.log('sessions', signClient?.session.getAll());

  useEffect(() => {
    SignClient.init({
      projectId: '7a11dea2c7ab88dc4597d5d44eb79a18',
      // optional parameters
      relayUrl: 'wss://relay.walletconnect.org',
      metadata: {
        name: 'Sage Wallet',
        description: 'Sage Wallet',
        url: 'https://sagewallet.com',
        icons: [
          'https://github.com/xch-dev/sage/blob/main/src-tauri/icons/icon.png?raw=true',
        ],
      },
    }).then((client) => {
      setSignClient(client);
    });
  }, []);

  const handleAndRespond = useCallback(
    async (request: SessionRequest) => {
      if (!signClient) throw new Error('Sign client not initialized');

      try {
        const result = await handleCommand(
          request.params.request.method as keyof typeof walletConnectCommands,
          request.params.request.params,
        );

        await signClient.respond({
          topic: request.topic,
          response: {
            id: request.id,
            jsonrpc: '2.0',
            result: result,
          },
        });
      } catch (e: any) {
        console.error(e);
        await signClient.respond({
          topic: request.topic,
          response: {
            id: request.id,
            jsonrpc: '2.0',
            error: e.message,
          },
        });
      }
    },
    [signClient],
  );

  useEffect(() => {
    if (!signClient) return;

    setSessions(signClient.session.getAll());

    async function handleSessionProposal(
      proposal: SignClientTypes.EventArguments['session_proposal'],
    ) {
      if (!signClient) throw new Error('Sign client not initialized');

      console.log('session proposal', proposal);
      console.log('active wallet', wallet);

      const {
        id: _id,
        params: {
          pairingTopic,
          proposer: { metadata: _proposerMetadata },
          requiredNamespaces,
        },
      } = proposal;

      if (!pairingTopic) {
        throw new Error('Pairing topic not found');
      }

      const requiredNamespace = requiredNamespaces.chia;
      if (!requiredNamespace) {
        throw new Error('Missing required chia namespace');
      }

      const { chains, methods, events } = requiredNamespace;
      const chain = chains?.find((item) =>
        ['chia:testnet', 'chia:mainnet'].includes(item),
      );
      if (!chain) {
        throw new Error('Chain not supported');
      }

      const networkConfig = await commands.networkConfig().catch(addError);

      if (!networkConfig) {
        throw new Error('Network config not found');
      }

      const network =
        networkConfig.network_id === 'mainnet' ? 'mainnet' : 'testnet';

      if (!wallet) {
        throw new Error('No active wallet');
      }

      const account = `chia:${network}:${wallet.fingerprint}`;
      const availableMethods = methods;
      const availableEvents = events;

      const { topic, acknowledged } = await signClient.approve({
        id: proposal.id,
        namespaces: {
          chia: {
            accounts: [account],
            methods: availableMethods,
            events: availableEvents,
          },
        },
      });
      console.log('topic', topic);

      await acknowledged();
      setSessions(signClient.session.getAll());
    }

    async function handleSessionRequest(request: SessionRequest) {
      const method = request.params.request
        .method as keyof typeof walletConnectCommands;

      console.log('session request', request);
      console.log(walletConnectCommands[method]);

      if (walletConnectCommands[method].confirm) {
        setPendingRequests((p: SessionRequest[]) => [...p, request]);
        await getCurrentWindow().requestUserAttention(
          UserAttentionType.Critical,
        );
      } else {
        await handleAndRespond(request);
      }
    }

    async function handleSessionDelete() {
      if (!signClient) throw new Error('Sign client not initialized');

      console.log('session deleted');
      setSessions(signClient.session.getAll());
    }

    signClient.on('session_proposal', handleSessionProposal);
    signClient.on('session_request', handleSessionRequest);
    signClient.on('session_delete', handleSessionDelete);
    return () => {
      signClient.off('session_proposal', handleSessionProposal);
      signClient.off('session_request', handleSessionRequest);
      signClient.off('session_delete', handleSessionDelete);
    };
  }, [signClient, wallet, handleAndRespond, setPendingRequests, addError]);

  const pair = async (uri: string) => {
    if (!signClient) throw new Error('Sign client not initialized');

    await signClient.core.pairing.pair({
      uri,
    });
  };

  const disconnect = async (topic: string) => {
    if (!signClient) throw new Error('Sign client not initialized');

    await signClient.disconnect({
      topic,
      reason: { code: 1, message: 'User disconnected' },
    });
    setSessions(signClient.session.getAll());
  };

  const approveRequest = async (request: SessionRequest) => {
    if (!pendingRequests.find((r) => r.id === request.id)) {
      return;
    }

    await handleAndRespond(request);
    setPendingRequests((p: SessionRequest[]) =>
      p.filter((r) => r.id !== request.id),
    );
  };

  const rejectRequest = async (request: SessionRequest) => {
    if (!signClient) throw new Error('Sign client not initialized');

    if (!pendingRequests.find((r) => r.id === request.id)) {
      return;
    }

    await signClient.respond({
      topic: request.topic,
      response: {
        id: request.id,
        jsonrpc: '2.0',
        result: null,
      },
    });
    setPendingRequests((p: SessionRequest[]) =>
      p.filter((r) => r.id !== request.id),
    );
  };

  return (
    <WalletConnectContext.Provider value={{ pair, sessions, disconnect }}>
      {children}
      {pendingRequests.length > 0 && (
        <RequestDialog
          request={pendingRequests[0]}
          approve={approveRequest}
          reject={rejectRequest}
          signClient={signClient}
        />
      )}
    </WalletConnectContext.Provider>
  );
}

interface RequestDialogProps {
  request: SessionRequest;
  approve: (request: SessionRequest) => void;
  reject: (request: SessionRequest) => void;
  signClient: InstanceType<typeof SignClient> | null;
}

interface CommandDialogProps<T extends WalletConnectCommand> {
  params: Params<T>;
}

function SignCoinSpendsDialog({
  params,
}: CommandDialogProps<'chip0002_signCoinSpends'>) {
  const [summary, setSummary] = useState<TransactionSummary | null>(null);
  const { addError } = useErrors();

  useEffect(() => {
    const coinSpends = params.coinSpends.map((coinSpend) => ({
      coin: {
        parent_coin_info: coinSpend.coin.parent_coin_info,
        puzzle_hash: coinSpend.coin.puzzle_hash,
        amount: coinSpend.coin.amount.toString(),
      },
      puzzle_reveal: coinSpend.puzzle_reveal,
      solution: coinSpend.solution,
    }));

    commands
      .viewCoinSpends({ coin_spends: coinSpends })
      .then((data) => setSummary(data.summary))
      .catch(addError);
  }, [params, addError]);

  return summary ? (
    <AdvancedSummary summary={summary} />
  ) : (
    <div className='p-4 text-center'>Loading transaction summary...</div>
  );
}

function SignMessageDialog({
  params,
}: CommandDialogProps<'chip0002_signMessage'>) {
  return (
    <div className='space-y-4 p-4'>
      <div className='space-y-2'>
        <div className='font-medium'>Public Key</div>
        <div className='text-sm text-muted-foreground break-all font-mono bg-muted p-2 rounded'>
          {params.publicKey}
        </div>
      </div>
      <div className='space-y-2'>
        <div className='font-medium'>Message</div>
        <div className='text-sm text-muted-foreground break-all font-mono bg-muted p-2 rounded whitespace-pre-wrap'>
          {params.message}
        </div>
      </div>
    </div>
  );
}

function TakeOfferDialog({ params }: CommandDialogProps<'chia_takeOffer'>) {
  const [offer, setOffer] = useState<OfferSummary | null>(null);
  const { addError } = useErrors();

  useEffect(() => {
    commands
      .viewOffer({ offer: params.offer })
      .then((data) => setOffer(data.offer))
      .catch(addError);
  }, [params, addError]);

  return offer ? (
    <OfferCard summary={offer} />
  ) : (
    <div className='p-4 text-center'>Loading offer details...</div>
  );
}

function CreateOfferDialog({ params }: CommandDialogProps<'chia_createOffer'>) {
  const walletState = useWalletState();

  return (
    <div className='space-y-4 p-4'>
      <div>
        <div className='font-medium mb-2'>Offering</div>
        <ul className='list-disc list-inside space-y-1'>
          {params.offerAssets.map((asset, i) => (
            <li key={i} className='text-sm'>
              {toDecimal(
                asset.amount,
                asset.assetId === '' ? walletState.sync.unit.decimals : 3,
              )}{' '}
              {asset.assetId || 'XCH'}
            </li>
          ))}
        </ul>
      </div>
      <div>
        <div className='font-medium mb-2'>Requesting</div>
        <ul className='list-disc list-inside space-y-1'>
          {params.requestAssets.map((asset, i) => (
            <li key={i} className='text-sm'>
              {toDecimal(
                asset.amount,
                asset.assetId === '' ? walletState.sync.unit.decimals : 3,
              )}{' '}
              {asset.assetId || 'XCH'}
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}

function DefaultCommandDialog({ params }: { params: unknown }) {
  return (
    <div className='p-4'>
      <div className='text-sm text-muted-foreground'>Command parameters:</div>
      <pre className='mt-2 rounded bg-muted p-4 overflow-auto'>
        <code className='text-xs'>{JSON.stringify(params, null, 2)}</code>
      </pre>
    </div>
  );
}

const COMMAND_COMPONENTS: {
  [K in WalletConnectCommand]?: (props: CommandDialogProps<K>) => JSX.Element;
} = {
  chip0002_signCoinSpends: SignCoinSpendsDialog,
  chip0002_signMessage: SignMessageDialog,
  chia_takeOffer: TakeOfferDialog,
  chia_createOffer: CreateOfferDialog,
};

const COMMAND_METADATA: {
  [K in WalletConnectCommand]?: {
    title: string;
    description: string;
  };
} = {
  chip0002_signCoinSpends: {
    title: 'Sign Transaction',
    description: 'Review and approve the transaction details below',
  },
  chip0002_signMessage: {
    title: 'Sign Message',
    description: 'Sign a message with your private key',
  },
  chia_takeOffer: {
    title: 'Accept Offer',
    description: 'Review and accept the offer',
  },
  chia_createOffer: {
    title: 'Create Offer',
    description: 'Review and create the offer',
  },
};

function RequestDialog({
  request,
  approve,
  reject,
  signClient,
}: RequestDialogProps) {
  const method = request.params.request.method as WalletConnectCommand;
  const params = request.params.request.params;
  const commandInfo = walletConnectCommands[method];
  const metadata = COMMAND_METADATA[method] ?? {
    title: 'Confirm Action',
    description: `Confirm ${method.replace(/_/g, ' ')}`,
  };
  const peerMetadata = signClient?.session.get(request.topic)?.peer.metadata;

  if (!commandInfo.confirm) {
    return null;
  }

  const CommandComponent = COMMAND_COMPONENTS[method] ?? DefaultCommandDialog;
  const parsedParams = commandInfo.paramsType.parse(params);

  return (
    <Dialog open={true} onOpenChange={(open) => !open && reject(request)}>
      <DialogContent className='max-w-2xl'>
        <DialogHeader>
          {peerMetadata && (
            <div className='text-sm text-muted-foreground mb-4'>
              Request from {peerMetadata.name}
            </div>
          )}
          <DialogTitle>{metadata.title}</DialogTitle>
          <DialogDescription>{metadata.description}</DialogDescription>
        </DialogHeader>

        <div className='max-h-[60vh] overflow-y-auto'>
          {CommandComponent && (
            <CommandComponent params={parsedParams as any} />
          )}
        </div>

        <DialogFooter className='sm:justify-between'>
          <DialogClose asChild>
            <Button variant='outline' onClick={() => reject(request)}>
              Reject
            </Button>
          </DialogClose>
          <Button onClick={() => approve(request)}>Approve</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
