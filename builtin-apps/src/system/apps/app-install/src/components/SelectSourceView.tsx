import { useState } from 'react';
import { AppModalShell } from '@sage-app/ui';
import { formatSageError } from '@sage-system-app/sdk';
import { closeSelf, previewUrl, selectAndPreviewZip } from '../api';
import { INSTALL_APP_ICON } from '../constants';
import type { InstallSource } from '../types';

export function SelectSourceView({
  onReview,
}: {
  onReview: (source: InstallSource) => void;
}) {
  const [urlInput, setUrlInput] = useState('');
  const [working, setWorking] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handlePreviewUrl() {
    setWorking(true);
    setError(null);

    try {
      onReview(await previewUrl(urlInput.trim()));
    } catch (err) {
      setError(formatSageError(err));
    } finally {
      setWorking(false);
    }
  }

  async function handleSelectZip() {
    setWorking(true);
    setError(null);

    try {
      const source = await selectAndPreviewZip();
      if (source) onReview(source);
    } catch (err) {
      setError(formatSageError(err));
    } finally {
      setWorking(false);
    }
  }

  return (
    <AppModalShell
      appName='Sage'
      appIcon={INSTALL_APP_ICON}
      title='Install app'
      footer={
        <div className='flex justify-end'>
          <button
            className='rounded-md border border-border px-4 py-2 text-sm disabled:opacity-60 hover:bg-muted'
            disabled={working}
            onClick={closeSelf}
          >
            Close
          </button>
        </div>
      }
    >
      <div className='space-y-5'>
        <div className='rounded-2xl border border-border p-4'>
          <div className='text-sm font-medium'>Install from URL</div>
          <p className='mt-1 text-sm text-muted-foreground'>
            Best for published apps and updates.
          </p>

          <div className='mt-4 flex gap-2'>
            <input
              className='min-w-0 flex-1 rounded-md border border-input bg-transparent px-3 py-2 text-sm outline-none'
              value={urlInput}
              disabled={working}
              placeholder='https://example.com/my-app/'
              onChange={(event) => setUrlInput(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === 'Enter' && urlInput.trim()) {
                  event.preventDefault();
                  void handlePreviewUrl();
                }
              }}
            />

            <button
              className='rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground disabled:opacity-60'
              disabled={working || !urlInput.trim()}
              onClick={handlePreviewUrl}
            >
              Preview
            </button>
          </div>
        </div>

        <div className='rounded-2xl border border-dashed border-border p-4'>
          <div className='text-sm font-medium'>Install from ZIP</div>
          <p className='mt-1 text-sm text-muted-foreground'>
            Useful for local builds, testing, or manual package installs.
          </p>

          <button
            className='mt-4 rounded-md border border-border px-4 py-2 text-sm disabled:opacity-60'
            disabled={working}
            onClick={handleSelectZip}
          >
            Select .zip package
          </button>
        </div>

        {error ? (
          <div className='rounded-xl border border-destructive/30 bg-destructive/10 p-3 text-sm text-destructive'>
            {error}
          </div>
        ) : null}
      </div>
    </AppModalShell>
  );
}
