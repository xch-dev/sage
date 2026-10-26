import { i18n } from '@lingui/core';
import { beforeAll, describe, expect, it } from 'vitest';
import {
  durationUntil,
  expirationDurationSeconds,
  expirationTimestamp,
  futureExpirationTimestamp,
  MAX_EXPIRATION_SECONDS,
} from './expiration';

const now = 1_800_000_000;

beforeAll(() => {
  i18n.loadAndActivate({ locale: 'en', messages: {} });
});

describe('expirationTimestamp', () => {
  it('counts a duration from now', () => {
    expect(
      expirationTimestamp({ days: '1', hours: '2', minutes: '3' }, now),
    ).toBe(now + 86_400 + 7_200 + 180);
  });

  it('uses a picked timestamp exactly, ignoring the duration', () => {
    expect(
      expirationTimestamp(
        { days: '9', hours: '9', minutes: '9', timestamp: now + 42 },
        now,
      ),
    ).toBe(now + 42);
  });

  it('treats blank or invalid fields as zero', () => {
    expect(
      expirationDurationSeconds({ days: '', hours: 'x', minutes: '5' }),
    ).toBe(300);
  });
});

describe('durationUntil', () => {
  it('splits the remaining time into days, hours and minutes', () => {
    expect(durationUntil(now + 86_400 + 7_200 + 180, now)).toEqual({
      days: '1',
      hours: '2',
      minutes: '3',
    });
  });

  it('rounds partial minutes up', () => {
    expect(durationUntil(now + 61, now)).toEqual({
      days: '0',
      hours: '0',
      minutes: '2',
    });
  });

  it('clamps past times to zero', () => {
    expect(durationUntil(now - 600, now)).toEqual({
      days: '0',
      hours: '0',
      minutes: '0',
    });
  });
});

describe('futureExpirationTimestamp', () => {
  it('returns the timestamp when it is in the future', () => {
    expect(
      futureExpirationTimestamp({ days: '0', hours: '0', minutes: '1' }, now),
    ).toBe(now + 60);
    expect(
      futureExpirationTimestamp(
        { days: '0', hours: '0', minutes: '0', timestamp: now + 1 },
        now,
      ),
    ).toBe(now + 1);
  });

  it('throws for a zero duration', () => {
    expect(() =>
      futureExpirationTimestamp({ days: '0', hours: '0', minutes: '0' }, now),
    ).toThrow();
  });

  it('throws for a negative duration', () => {
    expect(() =>
      futureExpirationTimestamp({ days: '1', hours: '-2', minutes: '0' }, now),
    ).toThrow();
  });

  it('throws for a picked time that is now or in the past', () => {
    expect(() =>
      futureExpirationTimestamp(
        { days: '1', hours: '0', minutes: '0', timestamp: now },
        now,
      ),
    ).toThrow();
  });

  it('throws for a duration that is too far in the future', () => {
    expect(() =>
      futureExpirationTimestamp(
        { days: '999999999999', hours: '0', minutes: '0' },
        now,
      ),
    ).toThrow('too far');
    expect(() =>
      futureExpirationTimestamp(
        {
          days: '0',
          hours: '0',
          minutes: '0',
          timestamp: now + MAX_EXPIRATION_SECONDS + 1,
        },
        now,
      ),
    ).toThrow('too far');
  });
});
