// @ts-check

import eslint from '@eslint/js';
import react from 'eslint-plugin-react';
import reactHooks from 'eslint-plugin-react-hooks';
import tseslint from 'typescript-eslint';

export default tseslint.config(
  {
    ignores: ['target/**/*', 'src/bindings.ts'],
  },
  {
    files: ['src/**/*.ts', 'src/**/*.tsx'],
    extends: [
      eslint.configs.recommended,
      tseslint.configs.recommended,
      tseslint.configs.strict,
      tseslint.configs.stylistic,
    ],
    plugins: {
      react,
      'react-hooks': reactHooks,
    },
    rules: {
      // React Hooks rules
      'react-hooks/rules-of-hooks': 'error',
      'react-hooks/exhaustive-deps': 'warn',
      'react/hook-use-state': 'warn',
    },
  },
  {
    settings: {
      react: {
        version: 'detect',
      },
    },
  },
);
