import {
  commands,
  OfferRecord,
  OfferSummary,
  TransactionSummary,
} from '@/bindings';
import { AdvancedTransactionSummary } from '@/components/AdvancedTransactionSummary';
import { OfferCard } from '@/components/OfferCard';
import { OfferSummaryCard } from '@/components/OfferSummaryCard';
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
import { useWallet } from '@/contexts/WalletContext';
import { useErrors } from '@/hooks/useErrors';
import { fromMojos, decodeHexMessage, isHex } from '@/lib/utils';
import { useWalletState } from '@/state';
import {
  Params,
  WalletConnectCommand,
  walletConnectCommands,
} from '@/walletconnect/commands';
import { handleCommand } from '@/walletconnect/handler';
import { getCurrentWindow, UserAttentionType } from '@tauri-apps/api/window';
import { platform } from '@tauri-apps/plugin-os';
import SignClient from '@walletconnect/sign-client';
import { SessionTypes, SignClientTypes } from '@walletconnect/types';
import {
  createContext,
  ReactNode,
  useCallback,
  useEffect,
  useMemo,
  useState,
} from 'react';
import { formatNumber } from '../i18n';
import { Switch } from '@/components/ui/switch';

export interface WalletConnectContextType {
  sessions: SessionTypes.Struct[];
  pair: (uri: string) => Promise<void>;
  disconnect: (topic: string) => Promise<void>;
  connecting: boolean;
}

export const WalletConnectContext = createContext<
  WalletConnectContextType | undefined
>(undefined);

type SessionRequest = SignClientTypes.EventArguments['session_request'];

export function WalletConnectProvider({ children }: { children: ReactNode }) {
  const { wallet } = useWallet();
  const { addError } = useErrors();

  const [signClient, setSignClient] = useState<Awaited<
    ReturnType<typeof SignClient.init>
  > | null>(null);
  const [sessions, setSessions] = useState<SessionTypes.Struct[]>([]);
  const [pendingRequests, setPendingRequests] = useState<SessionRequest[]>([]);
  const [connecting, setConnecting] = useState(false);

  useEffect(() => {
    SignClient.init({
      projectId: '7a11dea2c7ab88dc4597d5d44eb79a18',
      relayUrl: 'wss://relay.walletconnect.org',
      metadata: {
        name: 'Sage Wallet',
        description: 'Sage Wallet',
        url: 'https://sagewallet.net',
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
      if (!signClient) {
        console.error('Sign client not initialized');
        return;
      }

      try {
        const method = request.params.request
          .method as keyof typeof walletConnectCommands;
        const result = await handleCommand(
          method,
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
      } catch (error) {
        const errorMessage =
          error instanceof Error
            ? error.message
            : typeof error === 'object' && error !== null && 'reason' in error
              ? (error.reason as string)
              : 'Request failed';
        addError({ kind: 'walletconnect', reason: errorMessage });
        console.error('WalletConnect request failed:', error);

        await signClient.respond({
          topic: request.topic,
          response: {
            id: request.id,
            jsonrpc: '2.0',
            error: {
              code: 4001,
              message: errorMessage,
            },
          },
        });
      }
    },
    [signClient, addError],
  );

  useEffect(() => {
    if (!signClient) return;

    setSessions(signClient.session.getAll());

    async function handleSessionProposal(
      proposal: SignClientTypes.EventArguments['session_proposal'],
    ) {
      if (!signClient) {
        console.error('Sign client not initialized');
        return;
      }

      try {
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

        const network = await commands.getNetwork({});

        if (!wallet) {
          throw new Error('No active wallet');
        }

        const account = `chia:${network.kind}:${wallet.fingerprint}`;
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

        await acknowledged();
        setSessions(signClient.session.getAll());
        setConnecting(false);
      } catch (error) {
        const errorMessage =
          error instanceof Error ? error.message : 'Failed to connect';
        addError({ kind: 'walletconnect', reason: errorMessage });
        console.error('WalletConnect session proposal failed:', error);
        setConnecting(false);

        await signClient.reject({
          id: proposal.id,
          reason: {
            code: 4001,
            message: errorMessage,
          },
        });
      }
    }

    async function handleSessionRequest(request: SessionRequest) {
      try {
        const method = request.params.request
          .method as keyof typeof walletConnectCommands;

        if (!walletConnectCommands[method]) {
          throw new Error(`Unsupported method: ${method}`);
        }

        try {
          walletConnectCommands[method].paramsType.parse(
            request.params.request.params,
          );
        } catch (error) {
          console.error('Invalid parameters for method:', method, error);
          throw new Error(
            error instanceof Error
              ? error.message
              : `Invalid parameters for ${method}`,
          );
        }

        if (walletConnectCommands[method].confirm) {
          setPendingRequests((p: SessionRequest[]) => [...p, request]);
          const os = platform();
          if (os === 'macos' || os === 'windows' || os === 'linux') {
            await getCurrentWindow().requestUserAttention(
              UserAttentionType.Critical,
            );
          }
        } else {
          await handleAndRespond(request);
        }
      } catch (error) {
        console.error('WalletConnect session request failed:', error);

        if (signClient) {
          await signClient.respond({
            topic: request.topic,
            response: {
              id: request.id,
              jsonrpc: '2.0',
              error: {
                code: 4001,
                message:
                  error instanceof Error ? error.message : 'Request failed',
              },
            },
          });
        }
      }
    }

    async function handleSessionDelete() {
      if (!signClient) throw new Error('Sign client not initialized');

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
    if (!signClient) {
      console.error('Sign client not initialized');
      return;
    }

    try {
      setConnecting(true);
      await signClient.core.pairing.pair({ uri });
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : 'Failed to pair';
      addError({ kind: 'walletconnect', reason: errorMessage });
      console.error('WalletConnect pairing failed:', error);
      setConnecting(false);
    }
  };

  const disconnect = async (topic: string) => {
    if (!signClient) {
      console.error('Sign client not initialized');
      return;
    }

    try {
      await signClient.disconnect({
        topic,
        reason: { code: 4001, message: 'User disconnected' },
      });
      setSessions(signClient.session.getAll());
    } catch (error) {
      console.error('WalletConnect disconnect failed:', error);
    }
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
    <WalletConnectContext.Provider
      value={{ pair, sessions, disconnect, connecting }}
    >
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
    <AdvancedTransactionSummary summary={summary} />
  ) : (
    <div className='p-4 text-center'>Loading transaction summary...</div>
  );
}

function MessageToSign(params: { message: string }) {
  const [showDecoded, setShowDecoded] = useState(false);
  const isHexMessage = isHex(params.message);
  const message = isHexMessage
    ? !showDecoded
      ? params.message
      : decodeHexMessage(params.message)
    : params.message;

  return (
    <div className='space-y-2'>
      <div className='flex items-center justify-between gap-2 flex-wrap'>
        <div className='font-medium'>
          Message{' '}
          {isHexMessage && showDecoded && (
            <span className='text-xs text-muted-foreground ml-1'>
              (Decoded)
            </span>
          )}
        </div>
        {isHexMessage && (
          <div className='flex items-center gap-2'>
            <span className='text-sm text-muted-foreground whitespace-nowrap'>
              Show decoded
            </span>
            <Switch checked={showDecoded} onCheckedChange={setShowDecoded} />
          </div>
        )}
      </div>
      <div className='text-sm text-muted-foreground break-all font-mono bg-muted p-2 rounded whitespace-pre-wrap'>
        {message}
      </div>
    </div>
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
      <MessageToSign message={params.message} />
    </div>
  );
}

function SignMessageByAddressDialog({
  params,
}: CommandDialogProps<'chia_signMessageByAddress'>) {
  return (
    <div className='space-y-4 p-4'>
      <div className='space-y-2'>
        <div className='font-medium'>Address</div>
        <div className='text-sm text-muted-foreground break-all font-mono bg-muted p-2 rounded'>
          {params.address}
        </div>
      </div>
      <MessageToSign message={params.message} />
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
              {formatNumber({
                value: fromMojos(
                  asset.amount,
                  asset.assetId === '' ? walletState.sync.unit.decimals : 3,
                ),
                minimumFractionDigits: 0,
                maximumFractionDigits:
                  asset.assetId === '' ? walletState.sync.unit.decimals : 3,
              })}{' '}
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
              {formatNumber({
                value: fromMojos(
                  asset.amount,
                  asset.assetId === '' ? walletState.sync.unit.decimals : 3,
                ),
                minimumFractionDigits: 0,
                maximumFractionDigits:
                  asset.assetId === '' ? walletState.sync.unit.decimals : 3,
              })}{' '}
              {asset.assetId || 'XCH'}
            </li>
          ))}
        </ul>
      </div>
      <div>
        <div className='font-medium'>Fee</div>
        <div className='text-sm text-muted-foreground'>
          {formatNumber({
            value: fromMojos(params.fee || 0, walletState.sync.unit.decimals),
            minimumFractionDigits: 0,
            maximumFractionDigits: walletState.sync.unit.decimals,
          })}{' '}
          {walletState.sync.unit.ticker}
        </div>
      </div>
    </div>
  );
}

function CancelOfferDialog({ params }: CommandDialogProps<'chia_cancelOffer'>) {
  const walletState = useWalletState();
  const [record, setRecord] = useState<OfferRecord | null>(null);
  const { addError } = useErrors();

  useEffect(() => {
    commands
      .getOffer({ offer_id: params.id })
      .then((data) => setRecord(data.offer))
      .catch(addError);
  }, [params, addError]);

  return (
    <div className='space-y-2 p-4'>
      <div className='font-medium'>Offer ID</div>
      <div className='text-sm text-muted-foreground'>{params.id}</div>

      <div className='font-medium'>Fee</div>
      <div className='text-sm text-muted-foreground'>
        {formatNumber({
          value: fromMojos(params.fee || 0, walletState.sync.unit.decimals),
          minimumFractionDigits: 0,
          maximumFractionDigits: walletState.sync.unit.decimals,
        })}{' '}
        {walletState.sync.unit.ticker}
      </div>

      {record && (
        <div className='border rounded-md'>
          <OfferSummaryCard record={record} content={null} />
        </div>
      )}
    </div>
  );
}

function SendDialog({ params }: CommandDialogProps<'chia_send'>) {
  const walletState = useWalletState();

  return (
    <div className='space-y-2 p-4'>
      <div>
        <div className='font-medium'>Address</div>
        <div className='text-sm truncate text-muted-foreground'>
          {params.address}
        </div>
      </div>
      <div>
        <div className='font-medium'>Amount</div>
        <div className='text-sm text-muted-foreground'>
          {formatNumber({
            value: fromMojos(
              params.amount,
              params.assetId ? 3 : walletState.sync.unit.decimals,
            ),
            minimumFractionDigits: 0,
            maximumFractionDigits: params.assetId
              ? 3
              : walletState.sync.unit.decimals,
          })}{' '}
          {params.assetId ? 'CAT' : walletState.sync.unit.ticker}
        </div>
      </div>
      <div>
        <div className='font-medium'>Fee</div>
        <div className='text-sm text-muted-foreground'>
          {formatNumber({
            value: fromMojos(params.fee || 0, walletState.sync.unit.decimals),
            minimumFractionDigits: 0,
            maximumFractionDigits: walletState.sync.unit.decimals,
          })}{' '}
          {walletState.sync.unit.ticker}
        </div>
      </div>
      {params.assetId && (
        <div>
          <div className='font-medium'>Asset Id</div>
          <div className='text-sm text-muted-foreground'>{params.assetId}</div>
        </div>
      )}
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
  chia_cancelOffer: CancelOfferDialog,
  chia_send: SendDialog,
  chia_signMessageByAddress: SignMessageByAddressDialog,
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
  chia_cancelOffer: {
    title: 'Cancel Offer',
    description: 'Review and cancel the offer',
  },
  chia_signMessageByAddress: {
    title: 'Sign Message',
    description: "Sign a message with your address's private key",
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
    title: 'WalletConnect Request',
    description: `Would you like to authorize the "${method.split('_').slice(1).join(' ')}" request?`,
  };
  const peerMetadata = signClient?.session.get(request.topic)?.peer.metadata;

  const CommandComponent = COMMAND_COMPONENTS[method] ?? DefaultCommandDialog;

  const parsedParams = useMemo(
    () => commandInfo.paramsType.parse(params),
    [params, commandInfo],
  );

  if (!commandInfo.confirm) {
    return null;
  }

  return (
    <Dialog open={true} onOpenChange={(open) => !open && reject(request)}>
      <DialogContent className='max-w-2xl'>
        <DialogHeader>
          {peerMetadata && (
            <div className='text-sm text-muted-foreground mb-4'>
              From {peerMetadata.name}
            </div>
          )}
          <DialogTitle>{metadata.title}</DialogTitle>
          <DialogDescription>{metadata.description}</DialogDescription>
        </DialogHeader>

        <div className='max-h-[60vh] overflow-y-auto mb-2'>
          {CommandComponent && (
            <CommandComponent params={parsedParams as any} />
          )}
        </div>

        <DialogFooter>
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
