import { bech32m } from 'bech32';
import { describe, expect, it } from 'vitest';
import {
  buildSendLink,
  buildTakeOfferLink,
  isDeepLink,
  parseDeepLinkUrl,
} from './deepLink';

const XCH = 'xch1zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zygs3fazuv';
const TXCH = 'txch1zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zygsuw65al';
// A real (public) dexie.space offer, so the bech32m checksum is valid
const OFFER =
  'offer1qqr83wcuu2rykcmqvpsxygqqhqh7hu0l08gxt4gkk8ua3sl7h2hcrh66r69vezh0a6glrctp39vzvrznqah60rkkl63mtlmga5l447u06ml28d0ll7q64lwpshas9umanufaadfkrfhekxcex5slwjpvu0aqpuaedvp7xasxwmk6ayw442pz4kaa24dz2l7lc9du9t8mpnhhctqv83ekrpajjnwxrdhtulny0au5dr8cx4s45vks5g46l7xppxg9we4w00mwj5grcuxk8hx78a5uyrttvvkfm84r7hf4kvnls37yw0de5qv48s4eu7hg097cauyenuykzpa5v4hpk24wvkupuxn6hlev65a93kr3j78lxuekd8fxggwu33hue8w7y3erhm88k2recn4amqmxn02uawv3dyrp8trwddl9cwje72xtljy93w4j4jjv027ad8pdv6l6awl9wnduaaekvlkutac8d40hxmmztsls8q7d8nfm5cag4kvwr0zjpak0pp5x466f6hclr095thtch45a50nd4049qdkfn0j4mnvtfkunam2vvce9zlmy05emla8m0f8aumuhjuz8w7vqkkpq6pqkr5kscy6kfvprnahlzu79n4l2z7mctalqjtl75eudz6hvlly34s5j7ld4le829wzpwmrukmaqv6fvmlc9j2d678lcmwjhnau2urpzwznt23hgd8pamham5003e2gug9gd3cfddw2q2hr079uumal77mvve738hdkutu2hd8hkme0xyef6l0ft8pgazlnne0llglcl9ulld5m083phyceqq4vml3hez0vmegqqawvazchh73zy5c9rnm3av44tq5j94h3h0j3levnu7fkk84ct7f8s5rklx6el0036d9et5cmdsha7hy7wmszhq6h0hlm8hmk54804uhua7l4knyahwp6dfh2555w073vxdnznh6h7yyxhuwnc0z3xr6whl0j9a0e50zg7fyqy82ypdqgxvn78';
const ASSET_ID =
  'A628C1C2C6FCB74D53746157E438E108EAB5C0BB3E5C80FF9B1910B3E98A1E14';

describe('parseDeepLinkUrl', () => {
  describe('scheme and action', () => {
    it('rejects other schemes', () => {
      expect(parseDeepLinkUrl(`chia:send?address=${XCH}`)).toEqual({
        ok: false,
        error: 'invalid_scheme',
      });
    });

    it('accepts the scheme case-insensitively', () => {
      expect(parseDeepLinkUrl(`SAGE:send?address=${XCH}`).ok).toBe(true);
    });

    it('accepts the sage:// form and a trailing slash', () => {
      expect(parseDeepLinkUrl(`sage://send?address=${XCH}`).ok).toBe(true);
      expect(parseDeepLinkUrl(`sage://send/?address=${XCH}`).ok).toBe(true);
    });

    it('matches actions case-insensitively', () => {
      expect(parseDeepLinkUrl(`sage:TAKE_OFFER?offer=${OFFER}`).ok).toBe(true);
    });

    it('requires an action', () => {
      for (const url of ['sage:', 'sage://', `sage:?address=${XCH}`]) {
        expect(parseDeepLinkUrl(url)).toEqual({
          ok: false,
          error: 'missing_action',
        });
      }
    });

    it('reports unsupported actions by name', () => {
      expect(parseDeepLinkUrl('sage:add_offer?offer=x')).toEqual({
        ok: false,
        error: 'unsupported_action',
        action: 'add_offer',
      });
    });

    it('does not echo unprintable or prototype actions', () => {
      expect(parseDeepLinkUrl(`sage:${'a'.repeat(40)}`)).toEqual({
        ok: false,
        error: 'unsupported_action',
        action: undefined,
      });
      expect(parseDeepLinkUrl('sage:constructor').ok).toBe(false);
    });

    it('rejects the old payload-only forms', () => {
      expect(parseDeepLinkUrl(`sage:${OFFER}`).ok).toBe(false);
      expect(parseDeepLinkUrl(`sage:${XCH}?amount=1`).ok).toBe(false);
    });
  });

  describe('take_offer', () => {
    it('parses a raw offer with a fee', () => {
      expect(
        parseDeepLinkUrl(`sage:take_offer?offer=${OFFER}&fee=100`),
      ).toEqual({
        ok: true,
        link: { action: 'take_offer', offer: OFFER, fee: '100' },
      });
    });

    it('lowercases an all-uppercase offer', () => {
      const result = parseDeepLinkUrl(
        `sage:take_offer?offer=${OFFER.toUpperCase()}`,
      );
      expect(result.ok && result.link).toEqual({
        action: 'take_offer',
        offer: OFFER,
      });
    });

    it('rejects an offer whose checksum does not match', () => {
      const last = OFFER.at(-1) === 'q' ? 'p' : 'q';
      const corrupt = OFFER.slice(0, -1) + last;
      expect(parseDeepLinkUrl(`sage:take_offer?offer=${corrupt}`).ok).toBe(
        false,
      );
      expect(
        parseDeepLinkUrl(
          'sage:take_offer?offer=offer1qqzhthfamqu2eyjeuedycmzaqaqqxpea5vagelamekxa6',
        ).ok,
      ).toBe(false);
    });

    it('rejects malformed offers', () => {
      for (const offer of [
        'notanoffer',
        'Offer1Mixed',
        'offer1has-dash',
        `offer1${'q'.repeat(15000)}`,
      ]) {
        expect(parseDeepLinkUrl(`sage:take_offer?offer=${offer}`).ok).toBe(
          false,
        );
      }
    });

    it('parses an allowlisted offer url and canonicalizes it', () => {
      const url = encodeURIComponent(
        'https://dexie.space/offers/abc123/?ref=x#top',
      );
      expect(parseDeepLinkUrl(`sage:take_offer?url=${url}`)).toEqual({
        ok: true,
        link: {
          action: 'take_offer',
          offer: 'https://dexie.space/offers/abc123',
        },
      });
    });

    it('accepts each supported marketplace host', () => {
      for (const host of ['dexie.space', 'offerco.de', 'chia-offer.com']) {
        const url = encodeURIComponent(`https://${host}/offers/abc`);
        expect(parseDeepLinkUrl(`sage:take_offer?url=${url}`).ok).toBe(true);
      }
    });

    it('rejects urls outside the allowlist', () => {
      for (const raw of [
        'http://dexie.space/offers/abc',
        'https://evil.example/offers/abc',
        'https://dexie.space.evil.example/offers/abc',
        'https://www.dexie.space/offers/abc',
        'https://dexie.space/',
        'not a url',
      ]) {
        const url = encodeURIComponent(raw);
        expect(parseDeepLinkUrl(`sage:take_offer?url=${url}`).ok).toBe(false);
      }
    });

    it('rejects offer ids that are not plain tokens', () => {
      for (const raw of [
        'https://chia-offer.com/offers/abc&evil=1',
        'https://offerco.de/offers/a%26b=c',
        'https://dexie.space/offers/..%2F..%2Fv2%2Fx',
        'https://dexie.space/offers/abc.def',
        `https://dexie.space/offers/${'a'.repeat(129)}`,
      ]) {
        const url = encodeURIComponent(raw);
        // Literal & separator so the encoded value is not split
        expect(
          parseDeepLinkUrl(`sage:take_offer?fee=1&url=${url}`).ok,
          raw,
        ).toBe(false);
      }
    });

    it('requires exactly one of offer or url', () => {
      const url = encodeURIComponent('https://dexie.space/offers/abc');
      expect(parseDeepLinkUrl('sage:take_offer').ok).toBe(false);
      expect(
        parseDeepLinkUrl(`sage:take_offer?offer=${OFFER}&url=${url}`).ok,
      ).toBe(false);
    });

    it('drops a non-numeric fee', () => {
      const result = parseDeepLinkUrl(`sage:take_offer?offer=${OFFER}&fee=1.5`);
      expect(result.ok && result.link).toEqual({
        action: 'take_offer',
        offer: OFFER,
      });
    });
  });

  describe('send', () => {
    it('parses all parameters', () => {
      const url = `sage:send?address=${XCH}&amount=1000&fee=5&memos=hi%20there&asset_id=${ASSET_ID}`;
      expect(parseDeepLinkUrl(url)).toEqual({
        ok: true,
        link: {
          action: 'send',
          address: XCH,
          amount: '1000',
          fee: '5',
          memos: 'hi there',
          assetId: ASSET_ID.toLowerCase(),
        },
      });
    });

    it('accepts a testnet address', () => {
      expect(parseDeepLinkUrl(`sage:send?address=${TXCH}`).ok).toBe(true);
    });

    it('requires a valid address', () => {
      expect(parseDeepLinkUrl('sage:send?amount=1').ok).toBe(false);
      expect(parseDeepLinkUrl('sage:send?address=xch1nope').ok).toBe(false);
    });

    it('accepts any network prefix, leaving the network check to the wallet', () => {
      const dev =
        'dev1yg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3q6ucjwm';
      expect(parseDeepLinkUrl(`sage:send?address=${dev}`)).toEqual({
        ok: true,
        link: { action: 'send', address: dev },
      });
    });

    it('rejects addresses that are not a 32-byte puzzle hash', () => {
      const short =
        'xch1yg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zygtejw8s';
      expect(parseDeepLinkUrl(`sage:send?address=${short}`).ok).toBe(false);
    });

    it('rejects address prefixes that are not plain lowercase letters', () => {
      const odd =
        'xc-h1yg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3qlqh7ej';
      expect(parseDeepLinkUrl(`sage:send?address=${odd}`).ok).toBe(false);
    });

    it('drops amounts and fees longer than a u64', () => {
      const big = '9'.repeat(21);
      const url = `sage:send?address=${XCH}&amount=${big}&fee=${big}`;
      expect(parseDeepLinkUrl(url)).toEqual({
        ok: true,
        link: { action: 'send', address: XCH },
      });
      const overMax = parseDeepLinkUrl(
        `sage:send?address=${XCH}&amount=18446744073709551616`,
      );
      expect(overMax.ok && overMax.link).toEqual({
        action: 'send',
        address: XCH,
      });
      const max = '18446744073709551615';
      const result = parseDeepLinkUrl(`sage:send?address=${XCH}&amount=${max}`);
      expect(result.ok && result.link).toMatchObject({ amount: max });
    });

    it('drops invalid optional parameters', () => {
      const url = `sage:send?address=${XCH}&amount=-1&fee=abc&memos=${'m'.repeat(1001)}&asset_id=xyz`;
      expect(parseDeepLinkUrl(url)).toEqual({
        ok: true,
        link: { action: 'send', address: XCH },
      });
    });

    it.each(['nft', 'col'])('rejects a %s id as an address', (prefix) => {
      const id = bech32m.encode(prefix, bech32m.toWords(new Uint8Array(32)));
      expect(parseDeepLinkUrl(`sage:send?address=${id}`)).toEqual({
        ok: false,
        error: 'invalid_params',
      });
    });

    it('treats %26 as a separator when there is no literal &', () => {
      const result = parseDeepLinkUrl(
        `sage:send?address=${XCH}%26amount=10%26fee=2%26memos=hello`,
      );
      expect(result.ok && result.link).toEqual({
        action: 'send',
        address: XCH,
        amount: '10',
        fee: '2',
        memos: 'hello',
      });
    });

    it('keeps an encoded & inside a value when literal & separators are used', () => {
      const result = parseDeepLinkUrl(
        `sage:send?address=${XCH}&memos=salt%26pepper`,
      );
      expect(result.ok && result.link).toMatchObject({ memos: 'salt&pepper' });
    });
  });

  describe('install_app', () => {
    it('parses an app url', () => {
      expect(
        parseDeepLinkUrl('sage:install_app?url=https://xch.place'),
      ).toEqual({
        ok: true,
        link: { action: 'install_app', url: 'https://xch.place/' },
      });
    });

    it('keeps the path and drops the query string and fragment', () => {
      const result = parseDeepLinkUrl(
        `sage:install_app?url=${encodeURIComponent('https://example.com/apps/demo/?ref=x#top')}`,
      );
      expect(result.ok && result.link).toEqual({
        action: 'install_app',
        url: 'https://example.com/apps/demo/',
      });
    });

    it('shows an internationalized host as punycode', () => {
      const result = parseDeepLinkUrl('sage:install_app?url=https://xch.plаce');
      expect(result.ok && result.link).toMatchObject({
        url: 'https://xch.xn--plce-63d/',
      });
    });

    it.each([
      ['missing url', 'sage:install_app'],
      ['non-https url', 'sage:install_app?url=http://xch.place'],
      ['javascript url', 'sage:install_app?url=javascript:alert(1)'],
      ['bare host', 'sage:install_app?url=xch.place'],
      [
        'credentials disguising the host',
        `sage:install_app?url=${encodeURIComponent('https://xch.place@evil.example')}`,
      ],
      [
        'overlong url',
        `sage:install_app?url=https://example.com/${'a'.repeat(2048)}`,
      ],
    ])('rejects %s', (_, url) => {
      expect(parseDeepLinkUrl(url)).toEqual({
        ok: false,
        error: 'invalid_params',
      });
    });
  });
});

describe('link builders', () => {
  it('builds a send link the parser accepts', () => {
    expect(parseDeepLinkUrl(buildSendLink(XCH) ?? '')).toEqual({
      ok: true,
      link: { action: 'send', address: XCH },
    });
  });

  it('includes the asset id for a CAT', () => {
    expect(parseDeepLinkUrl(buildSendLink(XCH, ASSET_ID) ?? '')).toEqual({
      ok: true,
      link: { action: 'send', address: XCH, assetId: ASSET_ID.toLowerCase() },
    });
  });

  it('separates parameters with %26 so Android keeps the asset id', () => {
    const link = buildSendLink(XCH, ASSET_ID);
    expect(link).not.toContain('&');
    expect(link).toContain('%26asset_id=');
  });

  it('returns null for an address the parser would reject', () => {
    expect(buildSendLink('Unknown')).toBeNull();
    expect(buildSendLink(XCH, 'xyz')).toBeNull();
  });

  it('returns null for an offer too large for a link', () => {
    expect(buildTakeOfferLink(`offer1${'q'.repeat(15000)}`)).toBeNull();
  });

  it('builds a take_offer link the parser accepts', () => {
    expect(parseDeepLinkUrl(buildTakeOfferLink(OFFER) ?? '')).toEqual({
      ok: true,
      link: { action: 'take_offer', offer: OFFER },
    });
  });
});

describe('isDeepLink', () => {
  it.each([
    ['sage:send?address=x', true],
    ['  SAGE://take_offer', true],
    ['sage:', true],
    [XCH, false],
    ['https://dexie.space/offers/abc', false],
    ['', false],
  ])('%j → %s', (text, expected) => {
    expect(isDeepLink(text)).toBe(expected);
  });
});
