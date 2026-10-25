import { addressInfo } from '@/lib/utils';
import { bech32m } from 'bech32';

const SCHEME_PREFIX = 'sage:';

// Chia offers are bech32m encoded, max ~10KB when compressed
const MAX_OFFER_LENGTH = 15000;
const MAX_OFFER_URL_LENGTH = 2048;
const MAX_MEMO_LENGTH = 1000;

// Hosts resolveOfferData knows how to fetch an offer from
const OFFER_URL_HOSTS = ['dexie.space', 'offerco.de', 'chia-offer.com'];
const OFFER_ID_PATTERN = /^[A-Za-z0-9_-]{1,128}$/;

export interface TakeOfferDeepLink {
  action: 'take_offer';
  /** Either a raw `offer1...` string or a canonical marketplace offer URL. */
  offer: string;
  fee?: string;
}

export interface SendDeepLink {
  action: 'send';
  address: string;
  amount?: string;
  fee?: string;
  /** Named `memos` to match the `?memos=` query parameter the scheme accepts. */
  memos?: string;
  assetId?: string;
}

export type DeepLink = TakeOfferDeepLink | SendDeepLink;

export type DeepLinkError =
  | 'invalid_scheme'
  | 'missing_action'
  | 'unsupported_action'
  | 'invalid_params';

export type ParseResult =
  | { ok: true; link: DeepLink }
  | { ok: false; error: DeepLinkError; action?: string };

function decodeQueryString(queryString: string): URLSearchParams {
  // Android's Intent system truncates a URL at the first literal `&`, so senders
  // targeting Android encode the parameter separators as `%26` instead. Unescape
  // those separators only when the string actually has that shape - blanket
  // decoding a well-formed query string first would promote an encoded `&`, `+`
  // or `%` *inside* a value (a memo, typically) into a structural character and
  // silently corrupt it.
  if (!queryString.includes('&') && queryString.includes('%26')) {
    return new URLSearchParams(queryString.replace(/%26/g, '&'));
  }
  return new URLSearchParams(queryString);
}

function isMojos(value: string | null): value is string {
  // At most 20 digits, the width of a u64
  return !!value && /^\d{1,20}$/.test(value);
}

// Any network's address is accepted here; the link handler checks it against
// the active network so a mismatch gets its own message. The prefix is shown
// in that message, so it is held to plain lowercase letters.
function isWellFormedAddress(address: string): boolean {
  try {
    const { prefix, puzzleHash } = addressInfo(address);
    return puzzleHash.length === 64 && /^[a-z]{1,16}$/.test(prefix);
  } catch {
    return false;
  }
}

function parseOfferString(value: string): string | null {
  // bech32m is case-insensitive but must not be mixed case
  const offer = /^[A-Z0-9]+$/.test(value) ? value.toLowerCase() : value;
  if (
    !offer.startsWith('offer1') ||
    offer.length > MAX_OFFER_LENGTH ||
    !/^[a-z0-9]+$/.test(offer)
  ) {
    return null;
  }

  // Verify the checksum so a truncated or made-up offer is rejected here
  // rather than surfacing as an internal decode error on the offer page
  try {
    return bech32m.decode(offer, MAX_OFFER_LENGTH).prefix === 'offer'
      ? offer
      : null;
  } catch {
    return null;
  }
}

function parseOfferUrl(value: string): string | null {
  if (value.length > MAX_OFFER_URL_LENGTH) return null;

  let url: URL;
  try {
    url = new URL(value);
  } catch {
    return null;
  }

  if (url.protocol !== 'https:' || !OFFER_URL_HOSTS.includes(url.hostname)) {
    return null;
  }

  // resolveOfferData reads the offer id from the last path segment and
  // interpolates it unencoded into marketplace API requests, so it must be a
  // plain token. The query string and fragment are dropped for the same reason.
  const path = url.pathname.replace(/\/+$/, '');
  if (!OFFER_ID_PATTERN.test(path.split('/').pop() ?? '')) return null;

  return `https://${url.hostname}${path}`;
}

function parseTakeOffer(params: URLSearchParams): TakeOfferDeepLink | null {
  const offerParam = params.get('offer');
  const urlParam = params.get('url');

  // Exactly one source for the offer
  if (!offerParam === !urlParam) return null;

  const offer = offerParam
    ? parseOfferString(offerParam)
    : parseOfferUrl(urlParam as string);
  if (!offer) return null;

  const link: TakeOfferDeepLink = { action: 'take_offer', offer };
  const fee = params.get('fee');
  if (isMojos(fee)) link.fee = fee;
  return link;
}

function parseSend(params: URLSearchParams): SendDeepLink | null {
  const address = params.get('address');
  if (!address || !isWellFormedAddress(address)) return null;

  const link: SendDeepLink = { action: 'send', address };
  const amount = params.get('amount');
  const fee = params.get('fee');
  const memos = params.get('memos');
  const assetId = params.get('asset_id');

  if (isMojos(amount)) link.amount = amount;
  if (isMojos(fee)) link.fee = fee;
  if (memos && memos.length <= MAX_MEMO_LENGTH) link.memos = memos;
  // CAT asset IDs are 32-byte hex strings
  if (assetId && /^[0-9a-fA-F]{64}$/.test(assetId)) {
    link.assetId = assetId.toLowerCase();
  }
  return link;
}

const ACTIONS: Record<
  DeepLink['action'],
  (params: URLSearchParams) => DeepLink | null
> = {
  take_offer: parseTakeOffer,
  send: parseSend,
};

/** Parses `sage:<action>?<params>` (or `sage://<action>?<params>`). */
export function parseDeepLinkUrl(url: string): ParseResult {
  if (!url.toLowerCase().startsWith(SCHEME_PREFIX)) {
    return { ok: false, error: 'invalid_scheme' };
  }

  const payload = url.slice(SCHEME_PREFIX.length).replace(/^\/\//, '');
  const queryStart = payload.indexOf('?');
  const actionPart = queryStart === -1 ? payload : payload.slice(0, queryStart);
  const queryString = queryStart === -1 ? '' : payload.slice(queryStart + 1);

  const action = actionPart.replace(/\/+$/, '').toLowerCase();
  if (!action) {
    return { ok: false, error: 'missing_action' };
  }

  if (!Object.prototype.hasOwnProperty.call(ACTIONS, action)) {
    // Only echo the action back when it is short and printable
    return {
      ok: false,
      error: 'unsupported_action',
      action: /^[a-z0-9_-]{1,32}$/.test(action) ? action : undefined,
    };
  }

  const link = ACTIONS[action as DeepLink['action']](
    decodeQueryString(queryString),
  );
  return link ? { ok: true, link } : { ok: false, error: 'invalid_params' };
}
