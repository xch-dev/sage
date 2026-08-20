import { getSageSystemClient } from './runtime';

type SageSystemClientResolved = Awaited<ReturnType<typeof getSageSystemClient>>;

let sageSystemClient: SageSystemClientResolved | null = null;
let sageSystemClientPromise: Promise<SageSystemClientResolved> | null = null;

export function useSageSystemClient(): SageSystemClientResolved {
  if (sageSystemClient) return sageSystemClient;

  sageSystemClientPromise ??= getSageSystemClient().then((client) => {
    sageSystemClient = client;
    return client;
  });

  throw sageSystemClientPromise;
}
