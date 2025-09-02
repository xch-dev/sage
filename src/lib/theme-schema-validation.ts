import { Theme } from './theme.type';

export function validateTheme(data: unknown): Theme {
  if (typeof data !== 'object' || data === null)
    throw new Error(
      'Invalid theme JSON structure. The theme must be a valid JSON object',
    );
  const obj = data as Record<string, unknown>;
  if (typeof obj.name !== 'string')
    throw new Error('Invalid theme JSON structure. name is required');
  if (typeof obj.displayName !== 'string')
    throw new Error('Invalid theme JSON structure. displayName is required');
  if (typeof obj.schemaVersion !== 'number')
    throw new Error('Invalid theme JSON structure. schemaVersion is required');

  return data as Theme;
}
