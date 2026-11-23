import { describe, expect, it } from 'vitest';
import { databaseMigration } from './errors';

describe('databaseMigration', () => {
  it('tells which database failed to migrate', () => {
    expect(databaseMigration({ kind: 'database_migration', reason: '' })).toBe(
      'wallet',
    );
    expect(
      databaseMigration({ kind: 'shared_database_migration', reason: '' }),
    ).toBe('shared');
  });

  it('is null for other errors', () => {
    expect(databaseMigration({ kind: 'internal', reason: '' })).toBeNull();
    expect(databaseMigration('not an object')).toBeNull();
    expect(databaseMigration(null)).toBeNull();
  });
});
