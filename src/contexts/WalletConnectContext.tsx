import { createContext, ReactNode, useEffect, useState } from 'react';
import { SignClient } from '@walletconnect/sign-client';
import { useWallet } from '@/hooks/useWallet';
import useInitialization from '@/hooks/useInitialization';
import {
  Dialog,
  DialogDescription,
  DialogTitle,
  DialogHeader,
  DialogContent,
  DialogTrigger,
  DialogFooter,
  DialogClose,
} from '@/components/ui/dialog';
import { SessionTypes, SignClientTypes } from '@walletconnect/types';
import { Button } from '@/components/ui/button';
import { commands } from '@/bindings';
import {
  parseCommand as parseCommandParameters,
  walletConnectCommands,
} from '@/constants/walletConnectCommands';
import { getCurrentWindow, UserAttentionType } from '@tauri-apps/api/window';

const signClient = await SignClient.init({
  projectId: '681ef0ed0dd8de01da5e02d3299bc59d',
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
});

const handleWalletConnectCommand = async (
  command: keyof typeof walletConnectCommands,
  params: unknown,
) => {
  switch (command) {
    case 'chip0002_getPublicKeys':
      const { limit, offset } = parseCommandParameters(command, params);
      return commands.activeWallet().then((res) => {
        if (res.status === 'ok' && res.data) {
          return walletConnectCommands[command].returnType.parse([
            res.data.public_key,
          ]);
        }
        return null;
      });
    default:
      throw new Error(`Unknown command: ${command}`);
  }
};

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

  console.log('provider');

  console.log('sessions', signClient.session.getAll());

  const [sessions, setSessions] = useState<SessionTypes.Struct[]>([]);

  const [pendingRequests, setPendingRequests] = useState<SessionRequest[]>([]);

  async function handleAndRespond(request: SessionRequest) {
    try {
      const result = await handleWalletConnectCommand(
        request.params.request.method,
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
  }

  useEffect(() => {
    setSessions(signClient.session.getAll());

    async function handleSessionProposal(
      proposal: SignClientTypes.EventArguments['session_proposal'],
    ) {
      console.log('session proposal', proposal);
      console.log('active wallet', wallet);

      const {
        id,
        params: {
          pairingTopic,
          proposer: { metadata: proposerMetadata },
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

      const networkConfig = await commands.networkConfig().then((network) => {
        if (network.status === 'ok' && network.data) {
          return network.data;
        }
        return null;
      });

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
      console.log('session request', request);
      console.log(
        walletConnectCommands[
          request.params.request.method as keyof typeof walletConnectCommands
        ],
      );

      if (
        walletConnectCommands[
          request.params.request.method as keyof typeof walletConnectCommands
        ]?.requiresConfirmation
      ) {
        setPendingRequests((p: SessionRequest[]) => [...p, request]);
        await getCurrentWindow().requestUserAttention(
          UserAttentionType.Critical,
        );
      } else {
        await handleAndRespond(request);
      }
    }

    async function handleSessionDelete() {
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
  }, [wallet, setPendingRequests]);

  const pair = async (uri: string) => {
    await signClient.core.pairing.pair({
      uri,
    });
  };

  const disconnect = async (topic: string) => {
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
        <Dialog
          open={pendingRequests.length > 0}
          onOpenChange={(open) => !open && rejectRequest(pendingRequests[0])}
        >
          <DialogTrigger>Open</DialogTrigger>
          <DialogContent className='overflow-hidden'>
            <DialogHeader>
              <DialogTitle>WalletConnect Request</DialogTitle>
              <DialogDescription>
                {pendingRequests[0].params.request.method}
              </DialogDescription>
            </DialogHeader>
            <div className='mt-2 rounded bg-neutral-950 p-4 whitespace-pre break-words text-wrap overflow-auto max-h-[50vh]'>
              <code className='text-white text-xs'>
                {JSON.stringify(
                  pendingRequests[0].params.request.params,
                  null,
                  2,
                )}
              </code>
            </div>
            <DialogFooter>
              <DialogClose asChild>
                <Button variant='outline'>Reject</Button>
              </DialogClose>
              <Button onClick={() => approveRequest(pendingRequests[0])}>
                Approve
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      )}
    </WalletConnectContext.Provider>
  );
}
