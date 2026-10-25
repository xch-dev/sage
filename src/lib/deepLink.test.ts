import { describe, expect, it } from 'vitest';
import { parseDeepLinkUrl } from './deepLink';

const XCH = 'xch1zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zygs3fazuv';
const TXCH = 'txch1zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zygsuw65al';
const OFFER = 'offer1qqzhthfamqu2eyjeuedycmzaqaqqxpea5vagelamekxa6';
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

    it('drops invalid optional parameters', () => {
      const url = `sage:send?address=${XCH}&amount=-1&fee=abc&memos=${'m'.repeat(1001)}&asset_id=xyz`;
      expect(parseDeepLinkUrl(url)).toEqual({
        ok: true,
        link: { action: 'send', address: XCH },
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
});
