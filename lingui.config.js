import { defineConfig } from '@lingui/cli';
import { formatter } from '@lingui/format-po';

export default defineConfig({
  sourceLocale: 'en-US',
  locales: ['en-US', 'de-DE', 'zh-CN', 'es-MX'],
  format: formatter({ printPlaceholdersInComments: false }),

  catalogs: [
    {
      path: './src/locales/{locale}/messages',
      include: ['src'],
    },
  ],
});
