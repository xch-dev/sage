import { getSageClient, hasSageBridge } from './client';

type SageClientResolved = Awaited<ReturnType<typeof getSageClient>>;

let sageClient: SageClientResolved | null = null;
let sageClientPromise: Promise<SageClientResolved> | null = null;

export function useSageClient(): SageClientResolved {
  if (sageClient) return sageClient;

  if (!hasSageBridge()) {
    throw new Error('Sage bridge is not available');
  }

  sageClientPromise ??= getSageClient().then((client) => {
    sageClient = client;
    return client;
  });

  throw sageClientPromise;
}
