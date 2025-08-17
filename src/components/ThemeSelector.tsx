import { Button } from '@/components/ui/button';
import { useTheme } from '@/contexts/ThemeContext';
import { Trans } from '@lingui/react/macro';
import { Loader2 } from 'lucide-react';
import { ThemeCard } from './ThemeCard';

export function ThemeSelector() {
  const { currentTheme, setTheme, availableThemes, isLoading, error } =
    useTheme();

  if (isLoading) {
    return (
      <div className='flex items-center justify-center p-8'>
        <Loader2 className='h-6 w-6 animate-spin' />
        <span className='ml-2'>
          <Trans>Loading themes...</Trans>
        </span>
      </div>
    );
  }

  if (error) {
    return (
      <div className='text-center p-8 text-destructive'>
        <p>
          <Trans>Error loading themes: {error}</Trans>
        </p>
      </div>
    );
  }

  if (!currentTheme) {
    return (
      <div className='text-center p-8'>
        <Trans>No theme available</Trans>
      </div>
    );
  }

  return (
    <div className='grid grid-cols-2 md:grid-cols-3 gap-4'>
      {availableThemes.map((theme) => (
        <ThemeCard
          key={theme.name}
          theme={theme}
          isSelected={currentTheme.name === theme.name}
          onSelect={setTheme}
        />
      ))}
    </div>
  );
}

export function ThemeSelectorCompact() {
  const {
    currentTheme,
    setTheme,
    availableThemes,
    isLoading,
    lastUsedNonCoreTheme,
  } = useTheme();

  if (isLoading || !currentTheme) {
    return (
      <div className='flex items-center gap-2'>
        <Loader2 className='h-4 w-4 animate-spin' />
        <span className='text-sm'>Loading...</span>
      </div>
    );
  }

  // Get the core themes: light and dark
  const lightTheme = availableThemes.find((theme) => theme.name === 'light');
  const darkTheme = availableThemes.find((theme) => theme.name === 'dark');

  // Get the third theme: last used non-core theme or colorful as fallback
  let thirdTheme = null;
  if (lastUsedNonCoreTheme) {
    thirdTheme = availableThemes.find(
      (theme) => theme.name === lastUsedNonCoreTheme,
    );
  }

  // If no last used non-core theme or it's not available, use colorful as fallback
  if (!thirdTheme) {
    thirdTheme = availableThemes.find((theme) => theme.name === 'colorful');
  }

  const coreThemes = [lightTheme, darkTheme, thirdTheme].filter(
    (theme): theme is NonNullable<typeof theme> => theme !== undefined,
  );

  return (
    <div className='flex flex-wrap gap-2'>
      {coreThemes.map((theme) => (
        <Button
          key={theme.name}
          variant={currentTheme.name === theme.name ? 'default' : 'outline'}
          size='sm'
          onClick={() => setTheme(theme.name)}
          className='flex items-center gap-2'
          style={{ fontFamily: theme.fonts?.body || 'inherit' }}
        >
          <div
            className='w-3 h-3 rounded-full'
            style={{
              backgroundColor: theme.colors?.primary
                ? `hsl(${theme.colors.primary})`
                : undefined,
            }}
          />
          {theme.displayName}
        </Button>
      ))}
    </div>
  );
}

export function ThemeSelectorSimple() {
  const {
    currentTheme,
    setTheme,
    availableThemes,
    isLoading,
    lastUsedNonCoreTheme,
  } = useTheme();

  if (isLoading || !currentTheme) {
    return (
      <div className='space-y-3 p-4'>
        <div className='flex items-center justify-center'>
          <Loader2 className='h-4 w-4 animate-spin' />
          <span className='ml-2 text-sm'>Loading themes...</span>
        </div>
      </div>
    );
  }

  // Get the core themes: light and dark
  const lightTheme = availableThemes.find((theme) => theme.name === 'light');
  const darkTheme = availableThemes.find((theme) => theme.name === 'dark');

  // Get the third theme: last used non-core theme or colorful as fallback
  let thirdTheme = null;
  if (lastUsedNonCoreTheme) {
    thirdTheme = availableThemes.find(
      (theme) => theme.name === lastUsedNonCoreTheme,
    );
  }

  // If no last used non-core theme or it's not available, use colorful as fallback
  if (!thirdTheme) {
    thirdTheme = availableThemes.find((theme) => theme.name === 'colorful');
  }

  const coreThemes = [lightTheme, darkTheme, thirdTheme].filter(
    (theme): theme is NonNullable<typeof theme> => theme !== undefined,
  );

  return (
    <div className='space-y-3 p-4'>
      <div className='grid grid-cols-3 gap-3'>
        {coreThemes.map((theme) => (
          <ThemeCard
            key={theme.name}
            theme={theme}
            isSelected={currentTheme.name === theme.name}
            onSelect={setTheme}
            variant='simple'
          />
        ))}
      </div>
    </div>
  );
}
