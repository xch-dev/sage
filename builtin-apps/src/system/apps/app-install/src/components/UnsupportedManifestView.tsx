import { AppModalShell } from 'sage-app-ui';
import { closeSelf } from '../api';
import type { InstallSource } from '../types';

export function UnsupportedManifestView({
  source,
  error,
}: {
  source: InstallSource;
  error: string | null;
}) {
  const preview =
    source.kind === 'zip' ? source.preview : source.preview.manifest;
  const partial = preview.kind === 'partial' ? preview : null;

  return (
    <AppModalShell
      appName={partial?.manifest_header.name ?? 'Sage app'}
      title='App cannot be installed'
      footer={
        <div className='flex justify-end'>
          <button
            className='rounded-md border border-border px-4 py-2 text-sm'
            onClick={closeSelf}
          >
            Close
          </button>
        </div>
      }
    >
      <div className='space-y-4 text-sm'>
        {partial ? (
          <>
            <div className='text-muted-foreground'>
              Requires Sage {partial.manifest_header.sageVersion.min}
              {partial.manifest_header.sageVersion.testedMax
                ? ` · tested up to ${partial.manifest_header.sageVersion.testedMax}`
                : null}
            </div>

            <pre className='max-h-64 overflow-auto rounded-xl bg-muted p-3 text-xs whitespace-pre-wrap'>
              {partial.parse_error}
            </pre>
          </>
        ) : (
          <div className='text-destructive'>
            This app manifest cannot be installed by this Sage version.
          </div>
        )}

        {error ? (
          <div className='rounded-xl border border-destructive/30 bg-destructive/10 p-3 text-sm text-destructive'>
            {error}
          </div>
        ) : null}
      </div>
    </AppModalShell>
  );
}
