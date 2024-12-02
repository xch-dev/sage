import { Assets, commands, OfferSummary, TransactionSummary } from '@/bindings';
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
  DialogTrigger,
} from '@/components/ui/dialog';
import {
  parseCommand as parseCommandParameters,
  walletConnectCommands,
} from '@/constants/walletConnectCommands';
import useInitialization from '@/hooks/useInitialization';
import { useWallet } from '@/hooks/useWallet';
import { useWalletState } from '@/state';
import { getCurrentWindow, UserAttentionType } from '@tauri-apps/api/window';
import { SignClient } from '@walletconnect/sign-client';
import { SessionTypes, SignClientTypes } from '@walletconnect/types';
import BigNumber from 'bignumber.js';
import {
  createContext,
  ReactNode,
  useCallback,
  useEffect,
  useMemo,
  useState,
} from 'react';

const handleWalletConnectCommand = async (
  command: keyof typeof walletConnectCommands,
  params: unknown,
) => {
  switch (command) {
    case 'chip0002_chainId': {
      const networkConfig = await commands.networkConfig();

      if (networkConfig.status === 'error') {
        throw new Error(networkConfig.error.reason);
      }

      return networkConfig.data.network_id;
    }
    case 'chip0002_connect': {
      return true;
    }
    case 'chip0002_getPublicKeys': {
      const { limit, offset } = parseCommandParameters(command, params)!;

      const result = await commands.getDerivations({
        limit: limit ?? 10,
        offset: offset ?? 0,
      });

      if (result.status === 'error') {
        throw new Error(result.error.reason);
      }

      return result.data.derivations.map((derivation) => derivation.public_key);
    }
    case 'chip0002_filterUnlockedCoins': {
      const req = parseCommandParameters(command, params)!;

      const result = await commands.filterUnlockedCoins({
        coin_ids: req.coinNames,
      });

      if (result.status === 'error') {
        throw new Error(result.error.reason);
      }

      return result.data;
    }
    case 'chip0002_getAssetCoins': {
      const req = parseCommandParameters(command, params)!;

      const result = await commands.getAssetCoins(req);

      if (result.status === 'error') {
        throw new Error(result.error.reason);
      }

      return result.data;
    }
    case 'chip0002_getAssetBalance': {
      const req = parseCommandParameters(command, params)!;

      const result = await commands.getAssetCoins({
        ...req,
        includedLocked: true,
      });

      if (result.status === 'error') {
        throw new Error(result.error.reason);
      }

      let confirmed = BigNumber(0);
      let spendable = BigNumber(0);
      let spendableCoinCount = 0;

      for (const record of result.data) {
        confirmed = confirmed.plus(record.coin.amount);

        if (!record.locked) {
          spendable = spendable.plus(record.coin.amount);
          spendableCoinCount += 1;
        }
      }

      return {
        confirmed: confirmed.toString(),
        spendable: spendable.toString(),
        spendableCoinCount,
      };
    }
    case 'chip0002_signCoinSpends': {
      const req = parseCommandParameters(command, params)!;

      const result = await commands.signCoinSpends({
        coin_spends: req.coinSpends.map((coinSpend) => {
          return {
            coin: {
              parent_coin_info: coinSpend.coin.parent_coin_info,
              puzzle_hash: coinSpend.coin.puzzle_hash,
              amount: coinSpend.coin.amount.toString(),
            },
            puzzle_reveal: coinSpend.puzzle_reveal,
            solution: coinSpend.solution,
          };
        }),
        partial: req.partialSign,
        auto_submit: false,
      });

      if (result.status === 'error') {
        throw new Error(result.error.reason);
      }

      return result.data.spend_bundle.aggregated_signature;
    }
    case 'chip0002_signMessage': {
      const req = parseCommandParameters(command, params)!;

      const result = await commands.signMessageWithPublicKey(req);

      if (result.status === 'error') {
        throw new Error(result.error.reason);
      }

      return result.data.signature;
    }
    case 'chip0002_sendTransaction': {
      const req = parseCommandParameters(command, params)!;

      const result = await commands.sendTransactionImmediately({
        spend_bundle: req.spendBundle,
      });

      if (result.status === 'error') {
        throw new Error(result.error.reason);
      }

      return result.data;
    }
    case 'chia_createOffer': {
      const req = parseCommandParameters(command, params)!;
      const state = useWalletState.getState();

      const fee = BigNumber(req.fee ?? 0).div(
        BigNumber(10).pow(state.sync.unit.decimals),
      );

      const defaultAssets = (): Assets => {
        return {
          xch: '0',
          cats: [],
          nfts: [],
        };
      };

      const offerAssets = defaultAssets();
      const requestAssets = defaultAssets();

      for (const [from, to] of [
        [req.offerAssets, offerAssets],
        [req.requestAssets, requestAssets],
      ] as const) {
        for (const item of from) {
          if (item.assetId.startsWith('nft')) {
            to.nfts.push(item.assetId);
          } else if (item.assetId === '') {
            to.xch = BigNumber(to.xch)
              .plus(
                BigNumber(item.amount).div(
                  BigNumber(10).pow(state.sync.unit.decimals),
                ),
              )
              .toString();
          } else {
            const catAmount = BigNumber(item.amount).div(BigNumber(10).pow(3));
            const found = to.cats.find((cat) => cat.asset_id === item.assetId);

            if (found) {
              found.amount = BigNumber(found.amount).plus(catAmount).toString();
            } else {
              to.cats.push({
                asset_id: item.assetId,
                amount: catAmount.toString(),
              });
            }
          }
        }
      }

      const result = await commands.makeOffer({
        fee: fee.toString(),
        offered_assets: offerAssets,
        requested_assets: requestAssets,
        expires_at_second: null,
      });

      if (result.status === 'error') {
        throw new Error(result.error.reason);
      }

      return {
        offer: result.data.offer,
        id: result.data.offer_id,
      };
    }
    case 'chia_takeOffer': {
      const req = parseCommandParameters(command, params)!;
      const state = useWalletState.getState();

      const fee = BigNumber(req.fee ?? 0).div(
        BigNumber(10).pow(state.sync.unit.decimals),
      );

      const result = await commands.takeOffer({
        offer: req.offer,
        fee: fee.toString(),
        auto_submit: true,
      });

      if (result.status === 'error') {
        throw new Error(result.error.reason);
      }

      return { id: result.data.transaction_id };
    }
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
        const result = await handleWalletConnectCommand(
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
  }, [signClient, wallet, handleAndRespond, setPendingRequests]);

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
        />
      )}
    </WalletConnectContext.Provider>
  );
}

interface RequestDialogProps {
  request: SessionRequest;
  approve: (request: SessionRequest) => void;
  reject: (request: SessionRequest) => void;
}

function RequestDialog({ request, approve, reject }: RequestDialogProps) {
  const [summary, setSummary] = useState<TransactionSummary | null>(null);
  const [offer, setOffer] = useState<OfferSummary | null>(null);

  const method = useMemo(
    () => request.params.request.method as keyof typeof walletConnectCommands,
    [request],
  );

  const params = useMemo(() => request.params.request.params, [request]);

  const coinSpends = useMemo(
    () =>
      params.coinSpends?.map((coinSpend: any) => {
        return {
          coin: {
            parent_coin_info: coinSpend.coin.parent_coin_info,
            puzzle_hash: coinSpend.coin.puzzle_hash,
            amount: coinSpend.coin.amount.toString(),
          },
          puzzle_reveal: coinSpend.puzzle_reveal,
          solution: coinSpend.solution,
        };
      }) ?? [],
    [params],
  );

  useEffect(() => {
    if (method !== 'chip0002_signCoinSpends') {
      return;
    }

    setSummary(null);

    commands.viewCoinSpends({ coin_spends: coinSpends }).then((res) => {
      if (res.status === 'error') {
        reject(request);
        return;
      }

      setSummary(res.data.summary);
    });
  }, [coinSpends, request, method, reject]);

  useEffect(() => {
    if (method !== 'chia_takeOffer') {
      return;
    }

    setOffer(null);

    commands.viewOffer({ offer: params.offer }).then((res) => {
      if (res.status === 'error') {
        reject(request);
        return;
      }

      setOffer(res.data.offer);
    });
  }, [params.offer, request, method, reject]);

  console.log(request, summary);

  return (
    <Dialog open={true} onOpenChange={(open) => !open && reject(request)}>
      <DialogTrigger>Open</DialogTrigger>
      <DialogContent className='overflow-hidden'>
        <DialogHeader>
          <DialogTitle>WalletConnect Request</DialogTitle>
          <DialogDescription>{method}</DialogDescription>
        </DialogHeader>
        <div className='max-h-[50vh] overflow-y-scroll'>
          {summary !== null ? (
            <AdvancedSummary summary={summary} />
          ) : offer !== null ? (
            <OfferCard summary={offer} />
          ) : method === 'chip0002_signMessage' ? (
            <div>
              <div>Public Key</div>
              <div className='text-sm text-muted-foreground break-all'>
                {params.publicKey}
              </div>

              <div className='mt-3'>Message</div>
              <div className='text-sm text-muted-foreground break-all'>
                {params.message}
              </div>
            </div>
          ) : (
            <div>
              This is the raw request, since no summary has been displayed.
              <div className='mt-2 rounded bg-neutral-950 p-4 whitespace-pre break-words text-wrap'>
                <code className='text-white text-xs'>
                  {JSON.stringify(params, null, 2)}
                </code>
              </div>
            </div>
          )}
        </div>
        <DialogFooter>
          <DialogClose asChild>
            <Button variant='outline'>Reject</Button>
          </DialogClose>
          <Button onClick={() => approve(request)}>Approve</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
