import {
  ConnectionsSettings,
  matchesCredential,
} from '@/components/connections/ConnectionsSettings';
import { useConnections } from '@/contexts/ConnectionsContext';
import {
  commands,
  type AppPermissionReview,
  type AppPermissionReviews,
  type WalletInfo,
} from '@/bindings';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { useErrors } from '@/hooks/useErrors';
import { openAppPermissionsReview } from '@/lib/apps/openAppUpdate';
import { useWalletConnect } from '@/hooks/useWalletConnect';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { platform } from '@tauri-apps/plugin-os';
import type { SessionTypes } from '@walletconnect/types';
import {
  AppWindowIcon,
  ChevronDownIcon,
  GlobeIcon,
  RefreshCwIcon,
  ShieldCheckIcon,
} from 'lucide-react';
import {
  type ReactNode,
  useCallback,
  useEffect,
  useRef,
  useState,
} from 'react';
import { Link } from 'react-router-dom';

type Category = 'all' | 'apps' | 'external' | 'agents';

export function PermissionsSettings() {
  const connections = useConnections();
  const { sessions } = useWalletConnect();
  const isMobile = platform() === 'ios' || platform() === 'android';
  const [category, setCategory] = useState<Category>('all');
  const [search, setSearch] = useState('');
  const [apps, setApps] = useState<AppPermissionReviews | null>(null);
  const [wallets, setWallets] = useState<WalletInfo[]>([]);
  const [rpc, setRpc] = useState<boolean | null>(null);
  const [errors, setErrors] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);
  const [now, setNow] = useState(() => Date.now());
  const requestId = useRef(0);
  const invalidateRequests = useCallback(() => {
    ++requestId.current;
  }, []);

  const refresh = useCallback(async () => {
    const id = ++requestId.current;
    setLoading(true);
    const [appResult, walletResult, rpcResult] = await Promise.allSettled([
      isMobile
        ? Promise.resolve({ apps: [], unavailableAppIds: [] })
        : commands.appsReviewPermissions(),
      commands.getWallets({}),
      isMobile ? Promise.resolve(false) : commands.isRpcRunning(),
    ]);
    if (id !== requestId.current) return;
    const failures: string[] = [];
    if (appResult.status === 'fulfilled') setApps(appResult.value);
    else {
      setApps(null);
      failures.push(
        t`App permissions could not be loaded. Refresh to try again.`,
      );
    }
    if (walletResult.status === 'fulfilled')
      setWallets(walletResult.value.wallets);
    else {
      setWallets([]);
      failures.push(
        t`Wallet names could not be loaded. Wallet fingerprints are shown instead.`,
      );
    }
    if (rpcResult.status === 'fulfilled') setRpc(rpcResult.value);
    else {
      setRpc(null);
      failures.push(
        t`RPC status could not be loaded. Its access may still be enabled.`,
      );
    }
    setErrors(failures);
    setNow(Date.now());
    setLoading(false);
  }, [isMobile]);

  useEffect(() => {
    void refresh();
    const onFocus = () => void refresh();
    window.addEventListener('focus', onFocus);
    const timer = window.setInterval(() => setNow(Date.now()), 30_000);
    return () => {
      invalidateRequests();
      window.removeEventListener('focus', onFocus);
      window.clearInterval(timer);
    };
  }, [refresh, invalidateRequests]);

  const activeSessions = sessions.filter(
    (session) => session.expiry * 1000 > now,
  );
  const query = search.trim().toLowerCase();
  const appMatches = (app: AppPermissionReview) =>
    [
      app.name,
      app.id,
      ...app.capabilities.flatMap((cap) => [cap.label, cap.key]),
    ].some((value) => value.toLowerCase().includes(query));
  const visibleApps = (apps?.apps ?? []).filter(appMatches);
  const visibleSessions = activeSessions.filter((session) =>
    [
      session.peer.metadata.name,
      session.peer.metadata.url,
      ...Object.values(session.namespaces).flatMap((ns) => [
        ...ns.methods,
        ...ns.accounts,
      ]),
    ].some((value) => value.toLowerCase().includes(query)),
  );
  const showRpc =
    !isMobile &&
    [t`Full-access RPC certificate`, t`Full wallet access`, 'RPC'].some(
      (value) => value.toLowerCase().includes(query),
    );
  const credentials = connections?.view?.credentials ?? [];
  const agentCount = credentials.filter((c) => c.grant.kind === 'agent').length;
  const externalCount =
    activeSessions.length +
    (isMobile ? 0 : 1) +
    credentials.filter((c) => c.grant.kind === 'service').length;
  const count = (kind: Category): number | null =>
    kind === 'agents'
      ? agentCount
      : kind === 'external'
        ? externalCount
        : apps === null
          ? null
          : apps.apps.length +
            (kind === 'all' ? externalCount + agentCount : 0);
  const tabs: { id: Category; label: string }[] = [
    { id: 'all', label: t`All` },
    { id: 'apps', label: t`Apps` },
    { id: 'external', label: t`External services` },
    { id: 'agents', label: t`Agents` },
  ];

  const content = (kind: Category) => {
    const includeApps = kind === 'all' || kind === 'apps';
    const includeExternal = kind === 'all' || kind === 'external';
    const hasCredentials = credentials.some(
      (c) =>
        (kind === 'all' ||
          (kind === 'agents' && c.grant.kind === 'agent') ||
          (kind === 'external' && c.grant.kind === 'service')) &&
        matchesCredential(c, query),
    );
    const hasResults =
      hasCredentials ||
      (includeApps && visibleApps.length > 0) ||
      (includeExternal && (visibleSessions.length > 0 || showRpc));
    return (
      <div className='space-y-3'>
        {!isMobile && (
          <ConnectionsSettings
            category={kind}
            query={query}
            wallets={wallets}
          />
        )}
        {includeApps && isMobile && (
          <p className='text-sm text-muted-foreground'>
            <Trans>Sage Apps are available on desktop.</Trans>
          </p>
        )}
        {includeApps && (apps?.unavailableAppIds.length ?? 0) > 0 && (
          <Alert variant='warning'>
            <AlertDescription>
              <Trans>
                Some installed apps could not be read. Their permissions are not
                included below.
              </Trans>
              <ul className='mt-2 list-disc pl-5'>
                {apps?.unavailableAppIds.map((id) => (
                  <li className='break-all' key={id}>
                    {id}
                  </li>
                ))}
              </ul>
            </AlertDescription>
          </Alert>
        )}
        {includeApps &&
          visibleApps.map((app) => (
            <AppPermissions key={app.id} app={app} wallets={wallets} />
          ))}
        {includeExternal &&
          visibleSessions.map((session) => (
            <SessionPermissions key={session.topic} session={session} />
          ))}
        {includeExternal && showRpc && (
          <PermissionCard
            icon={<ShieldCheckIcon />}
            name={t`Full-access RPC certificate`}
            kind={t`External service`}
            summary={t`Full wallet access · All wallets`}
          >
            <div className='space-y-3 text-sm'>
              <Badge variant='secondary'>
                {rpc === null
                  ? t`Status unavailable`
                  : rpc
                    ? t`Server running`
                    : t`Server stopped`}
              </Badge>
              <p>
                <Trans>
                  The owner RPC certificate grants full access to wallet
                  operations and settings. Individual clients using this
                  certificate cannot be listed separately.
                </Trans>
              </p>
              <p className='text-muted-foreground'>
                <Trans>
                  Stopping the server blocks connections but does not revoke the
                  certificate.
                </Trans>
              </p>
              <Button variant='outline' size='sm' asChild>
                <Link to='/settings/connections'>
                  <Trans>Manage RPC server</Trans>
                </Link>
              </Button>
            </div>
          </PermissionCard>
        )}
        {!loading && !hasResults && !(includeApps && apps === null) && (
          <Empty
            icon={<ShieldCheckIcon />}
            title={
              query
                ? t`No matching permissions`
                : kind === 'agents'
                  ? t`No agent connections yet`
                  : t`No connections to review`
            }
            description={
              query
                ? t`Try a different name or permission.`
                : t`Granted permissions will appear here when you connect an app or service.`
            }
          />
        )}
      </div>
    );
  };

  return (
    <div className='space-y-5'>
      <div className='flex items-start justify-between gap-3'>
        <div className='space-y-1'>
          <h3 className='font-medium'>
            <Trans>Who has access</Trans>
          </h3>
          <p className='text-sm text-muted-foreground'>
            <Trans>
              Review access granted to installed apps and connected services,
              across all wallets.
            </Trans>
          </p>
        </div>
        <Button
          variant='outline'
          size='sm'
          onClick={() => void refresh()}
          disabled={loading}
          aria-label={t`Refresh permissions`}
        >
          <RefreshCwIcon
            className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`}
          />
        </Button>
      </div>
      {errors.map((error) => (
        <Alert variant='warning' key={error}>
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      ))}
      <Tabs
        value={category}
        onValueChange={(value) => setCategory(value as Category)}
      >
        <div className='overflow-x-auto pb-1'>
          <TabsList aria-label={t`Permission categories`} className='h-auto'>
            {tabs.map(({ id, label }) => (
              <TabsTrigger key={id} value={id}>
                {label}
                <span className='ml-2 text-xs tabular-nums text-muted-foreground'>
                  {count(id) ?? '—'}
                </span>
              </TabsTrigger>
            ))}
          </TabsList>
        </div>
        <Input
          className='my-4'
          value={search}
          onChange={(event) => setSearch(event.target.value)}
          placeholder={t`Search names or permissions`}
          aria-label={t`Search permissions`}
        />
        {loading && (
          <p role='status' className='mb-3 text-sm text-muted-foreground'>
            <Trans>Loading permissions…</Trans>
          </p>
        )}
        {tabs.map(({ id }) => (
          <TabsContent key={id} value={id}>
            {content(id)}
          </TabsContent>
        ))}
      </Tabs>
      <p className='text-xs text-muted-foreground'>
        <Trans>
          This view shows granted access, not a history of actions. Built-in
          Sage components are not listed.
        </Trans>
      </p>
    </div>
  );
}

function PermissionCard({
  icon,
  name,
  kind,
  summary,
  children,
}: {
  icon: ReactNode;
  name: string;
  kind: string;
  summary: string;
  children: ReactNode;
}) {
  return (
    <details className='group rounded-lg border bg-card'>
      <summary className='flex cursor-pointer list-none items-center gap-3 rounded-lg p-4 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring [&::-webkit-details-marker]:hidden'>
        <span className='rounded-md bg-muted p-2 [&>svg]:h-5 [&>svg]:w-5'>
          {icon}
        </span>
        <div className='min-w-0 flex-1'>
          <div className='flex flex-wrap items-center gap-2'>
            <span className='break-words font-medium'>{name}</span>
            <Badge variant='secondary'>{kind}</Badge>
          </div>
          <p className='mt-1 text-sm text-muted-foreground'>{summary}</p>
        </div>
        <ChevronDownIcon className='h-4 w-4 shrink-0 text-muted-foreground transition-transform group-open:rotate-180' />
      </summary>
      <div className='space-y-5 border-t p-4'>{children}</div>
    </details>
  );
}

function AppPermissions({
  app,
  wallets,
}: {
  app: AppPermissionReview;
  wallets: WalletInfo[];
}) {
  const { addError } = useErrors();
  const walletScope =
    app.walletScope.kind === 'allWallets'
      ? t`All wallets`
      : app.walletScope.fingerprints.length === 0
        ? t`No wallets`
        : app.walletScope.fingerprints
            .map((fingerprint) => {
              const name = wallets.find(
                (wallet) => wallet.fingerprint === fingerprint,
              )?.name;
              return name ? `${name} (${fingerprint})` : String(fingerprint);
            })
            .join(', ');
  const network = [
    ...app.network.whitelist.map((entry) => ({
      ...entry,
      network: t`All networks`,
    })),
    ...Object.entries(app.network.whitelistByNetwork ?? {}).flatMap(
      ([network, entries]) =>
        (entries ?? []).map((entry) => ({ ...entry, network })),
    ),
  ];
  return (
    <PermissionCard
      icon={<AppWindowIcon />}
      name={app.name}
      kind={t`App`}
      summary={walletScope}
    >
      <div>
        <p className='text-xs text-muted-foreground'>
          <Trans>App identity</Trans>
        </p>
        <p className='mt-1 break-all font-mono text-xs'>{app.id}</p>
      </div>
      {[true, false].map((wallet) => (
        <section key={String(wallet)} className='space-y-2'>
          <h4 className='text-sm font-medium'>
            {wallet ? (
              <Trans>Wallet permissions</Trans>
            ) : (
              <Trans>App permissions</Trans>
            )}
          </h4>
          {app.capabilities.some((cap) => cap.wallet === wallet) ? (
            <ul className='divide-y'>
              {app.capabilities
                .filter((cap) => cap.wallet === wallet)
                .map((cap) => (
                  <li key={cap.key} className='py-2 text-sm'>
                    <div className='flex flex-wrap items-center gap-2'>
                      <span>{cap.label}</span>
                      {cap.automatic && (
                        <Badge variant='outline'>
                          <Trans>Included with app</Trans>
                        </Badge>
                      )}
                    </div>
                    <p className='mt-1 text-xs text-muted-foreground'>
                      {cap.description}
                    </p>
                    <code className='text-xs text-muted-foreground'>
                      {cap.key}
                    </code>
                  </li>
                ))}
            </ul>
          ) : (
            <p className='text-sm text-muted-foreground'>
              <Trans>None granted</Trans>
            </p>
          )}
        </section>
      ))}
      <section className='space-y-2'>
        <h4 className='text-sm font-medium'>
          <Trans>Network access</Trans>
        </h4>
        {network.length ? (
          <ul className='space-y-2'>
            {network.map((entry) => (
              <li
                key={`${entry.network}:${entry.scheme}:${entry.host}`}
                className='flex flex-wrap justify-between gap-2 text-sm'
              >
                <code className='break-all'>
                  {entry.scheme}://{entry.host}
                </code>
                <span className='text-xs text-muted-foreground'>
                  {entry.network}
                </span>
              </li>
            ))}
          </ul>
        ) : (
          <p className='text-sm text-muted-foreground'>
            <Trans>No network destinations granted</Trans>
          </p>
        )}
      </section>
      <Button
        variant='outline'
        size='sm'
        onClick={() => {
          void openAppPermissionsReview(app.id).catch(addError);
        }}
      >
        <Trans>Manage app permissions</Trans>
      </Button>
    </PermissionCard>
  );
}

function SessionPermissions({ session }: { session: SessionTypes.Struct }) {
  const namespaces = Object.values(session.namespaces);
  const methods = [...new Set(namespaces.flatMap((ns) => ns.methods))].sort();
  const accounts = [...new Set(namespaces.flatMap((ns) => ns.accounts))];
  const events = [...new Set(namespaces.flatMap((ns) => ns.events))].sort();
  return (
    <PermissionCard
      icon={<GlobeIcon />}
      name={session.peer.metadata.name || t`Unnamed service`}
      kind='WalletConnect'
      summary={session.peer.metadata.url}
    >
      <p className='text-sm text-muted-foreground'>
        <Trans>
          This connection uses WalletConnect’s session and approval flow.
        </Trans>
      </p>
      <section className='space-y-2'>
        <h4 className='text-sm font-medium'>
          <Trans>Session methods</Trans>
        </h4>
        <ul className='space-y-1 text-xs'>
          {methods.map((method) => (
            <li key={method}>
              <code>{method}</code>
            </li>
          ))}
        </ul>
        {methods.length === 0 && (
          <p className='text-sm text-muted-foreground'>
            <Trans>None granted</Trans>
          </p>
        )}
      </section>
      <section className='space-y-2'>
        <h4 className='text-sm font-medium'>
          <Trans>Accounts</Trans>
        </h4>
        <ul className='space-y-1 text-xs'>
          {accounts.map((account) => (
            <li className='break-all font-mono' key={account}>
              {account}
            </li>
          ))}
        </ul>
      </section>
      {events.length > 0 && (
        <section className='space-y-2'>
          <h4 className='text-sm font-medium'>
            <Trans>Session events</Trans>
          </h4>
          <ul className='space-y-1 text-xs'>
            {events.map((event) => (
              <li key={event}>
                <code>{event}</code>
              </li>
            ))}
          </ul>
        </section>
      )}
      <p className='text-xs text-muted-foreground'>
        <Trans>Expires</Trans>:{' '}
        {new Date(session.expiry * 1000).toLocaleString()}
      </p>
      <Button variant='outline' size='sm' asChild>
        <Link to='/settings/connections'>
          <Trans>Manage connection</Trans>
        </Link>
      </Button>
    </PermissionCard>
  );
}

function Empty({
  icon,
  title,
  description,
}: {
  icon: ReactNode;
  title: string;
  description: string;
}) {
  return (
    <div className='rounded-lg border border-dashed px-5 py-10 text-center'>
      <div className='mx-auto mb-3 w-fit text-muted-foreground'>{icon}</div>
      <p className='text-sm font-medium'>{title}</p>
      <p className='mx-auto mt-2 max-w-md text-sm text-muted-foreground'>
        {description}
      </p>
    </div>
  );
}
