import { useLocalStorage } from 'usehooks-ts';

/**
 * Whether new offers split coins first by default. On unless turned off, since
 * without a split an offer can lock up more of the balance than it offers.
 */
export function useDefaultOfferSplit() {
  const [splitByDefault, setSplitByDefault] = useLocalStorage<boolean>(
    'default-offer-split',
    true,
  );

  return { splitByDefault, setSplitByDefault };
}
