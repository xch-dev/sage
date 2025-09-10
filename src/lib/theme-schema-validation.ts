import { Theme } from './theme.type';

export function validateTheme(data: unknown): Theme {
  let parsedData: unknown = data;

  // If data is a string, try to parse it as JSON
  if (typeof data === 'string') {
    try {
      parsedData = JSON.parse(data);
    } catch {
      throw new Error(
        'Invalid theme JSON structure. The theme string must be valid JSON',
      );
    }
  }

  if (typeof parsedData !== 'object' || parsedData === null)
    throw new Error(
      'Invalid theme JSON structure. The theme must be a valid JSON object',
    );
  const obj = parsedData as Record<string, unknown>;
  if (typeof obj.name !== 'string')
    throw new Error('Invalid theme JSON structure. name is required');
  if (typeof obj.displayName !== 'string')
    throw new Error('Invalid theme JSON structure. displayName is required');
  if (typeof obj.schemaVersion !== 'number')
    throw new Error('Invalid theme JSON structure. schemaVersion is required');

  const theme = parsedData as Theme;
  if (theme.schemaVersion !== 1)
    throw new Error(
      `Invalid theme JSON structure. Unrecognized schemaVersion: ${theme.schemaVersion}`,
    );

  if (!theme.tags) {
    theme.tags = [];
  }

  return theme;
}
