import { describe, expect, it } from 'vitest';
import {
  durationUntil,
  expirationDurationSeconds,
  expirationTimestamp,
} from './expiration';

const now = 1_800_000_000;

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
