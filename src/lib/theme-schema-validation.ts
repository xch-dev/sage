import Ajv, { type ErrorObject } from 'ajv';
import themeSchema from '../themes/schema.json';
import { Theme } from './theme.type';

// Create Ajv instance
const ajv = new Ajv({
  allErrors: false, // Stop at first validation error
  verbose: false, // Include schema and data in errors for better debugging
});

// Compile the theme schema once for performance
const validateThemeSchema = ajv.compile(themeSchema);

export function validateThemeJson(themeJson: string): {
  success: boolean;
  data?: Theme;
  error?: string;
  details?: ErrorObject[];
} {
  try {
    const parsedTheme = JSON.parse(themeJson);
    return validateTheme(parsedTheme);
  } catch (error) {
    return {
      success: false,
      error: `Invalid JSON: ${error instanceof Error ? error.message : String(error)}`,
    };
  }
}

export function validateTheme(themeData: unknown): {
  success: boolean;
  data?: Theme;
  error?: string;
  details?: ErrorObject[];
} {
  try {
    const isValid = validateThemeSchema(themeData);

    if (isValid) {
      return {
        success: true,
        data: themeData as unknown as Theme,
      };
    } else {
      const errors = validateThemeSchema.errors || [];
      const errorMessages = errors.map((err) => {
        const path = err.instancePath || 'root';
        let message = `${path}: ${err.message}`;

        // Add more context for specific error types
        if (err.keyword === 'required') {
          message = `Missing required property: ${err.params?.missingProperty || 'unknown'}`;
        } else if (err.keyword === 'type') {
          message = `${path}: Expected ${err.params?.type}, but got ${typeof err.data}`;
        } else if (err.keyword === 'enum') {
          message = `${path}: Must be one of [${err.params?.allowedValues?.join(', ') || 'unknown'}]`;
        }

        return message;
      });

      return {
        success: false,
        error: `Theme validation failed:\n${errorMessages.join('\n')}`,
        details: errors,
      };
    }
  } catch (error) {
    return {
      success: false,
      error: `Unexpected validation error: ${error instanceof Error ? error.message : String(error)}`,
    };
  }
}
