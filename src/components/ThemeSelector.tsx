import { Button } from '@/components/ui/button';
import { useTheme } from '@/contexts/ThemeContext';
import { Trans } from '@lingui/react/macro';
import { Check, Loader2 } from 'lucide-react';

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
        <div
          key={theme.name}
          className={`cursor-pointer transition-all hover:opacity-90 ${
            currentTheme.name === theme.name ? 'ring-2' : 'hover:ring-1'
          }`}
          style={{
            backgroundColor: theme.backgroundImage
              ? `hsla(${theme.colors?.card || 'var(--card)'}, ${theme.backgroundOpacity?.card ?? 0.75})`
              : theme.colors?.card
                ? `hsl(${theme.colors.card})`
                : 'hsl(var(--card))',
            color: theme.colors?.cardForeground
              ? `hsl(${theme.colors.cardForeground})`
              : 'hsl(var(--card-foreground))',
            border: theme.colors?.border
              ? `1px solid hsl(${theme.colors.border})`
              : '1px solid hsl(var(--border))',
            borderRadius: theme.corners?.lg || 'var(--corner-lg)',
            boxShadow: theme.shadows?.card || 'var(--shadow-card)',
            fontFamily: theme.fonts?.body || 'var(--font-body)',
            backgroundImage: theme.backgroundImage
              ? `url(${theme.backgroundImage})`
              : undefined,
            backgroundSize: theme.backgroundImage ? 'cover' : undefined,
            backgroundPosition: theme.backgroundImage ? 'center' : undefined,
            outline:
              currentTheme.name === theme.name
                ? `2px solid ${currentTheme.colors?.primary ? `hsl(${currentTheme.colors.primary})` : 'hsl(var(--primary))'}`
                : 'none',
          }}
          onClick={() => setTheme(theme.name)}
        >
          <div className='p-4'>
            <div className='flex items-center justify-between mb-3'>
              <h3
                className='font-medium text-sm'
                style={{ fontFamily: theme.fonts?.heading || 'var(--font-heading)' }}
              >
                {theme.displayName}
              </h3>
              {currentTheme.name === theme.name && (
                <Check
                  className='h-4 w-4'
                  style={{
                    color: currentTheme.colors?.primary
                      ? `hsl(${currentTheme.colors.primary})`
                      : 'hsl(var(--primary))',
                  }}
                />
              )}
            </div>

            {/* Theme preview */}
            <div className='space-y-2'>
              <div
                className='h-8 flex items-center px-2'
                style={{
                  backgroundColor: theme.colors?.primary
                    ? `hsl(${theme.colors.primary})`
                    : 'hsl(var(--primary))',
                  borderColor: theme.colors?.border
                    ? `hsl(${theme.colors.border})`
                    : 'hsl(var(--border))',
                  color: theme.colors?.primaryForeground
                    ? `hsl(${theme.colors.primaryForeground})`
                    : 'hsl(var(--primary-foreground))',
                  fontFamily: theme.fonts?.heading || 'var(--font-heading)',
                  borderRadius: theme.corners?.md || 'var(--corner-md)',
                  border: theme.colors?.border
                    ? `1px solid hsl(${theme.colors.border})`
                    : '1px solid hsl(var(--border))',
                  boxShadow: theme.shadows?.button || 'var(--shadow-button)',
                }}
              >
                <span className='text-xs font-medium'>Aa</span>
              </div>
              <div className='flex gap-1'>
                <div
                  className='h-4 w-4'
                  style={{
                    backgroundColor: theme.colors?.primary
                      ? `hsl(${theme.colors.primary})`
                      : undefined,
                    borderRadius: theme.corners?.sm || '0.125rem',
                  }}
                />
                <div
                  className='h-4 w-4'
                  style={{
                    backgroundColor: theme.colors?.secondary
                      ? `hsl(${theme.colors.secondary})`
                      : undefined,
                    borderRadius: theme.corners?.sm || '0.125rem',
                  }}
                />
                <div
                  className='h-4 w-4'
                  style={{
                    backgroundColor: theme.colors?.accent
                      ? `hsl(${theme.colors.accent})`
                      : undefined,
                    borderRadius: theme.corners?.sm || '0.125rem',
                  }}
                />
                <div
                  className='h-4 w-4'
                  style={{
                    backgroundColor: theme.colors?.destructive
                      ? `hsl(${theme.colors.destructive})`
                      : undefined,
                    borderRadius: theme.corners?.sm || '0.125rem',
                  }}
                />
              </div>
              <div
                className='text-xs truncate'
                style={{
                  color: theme.colors?.mutedForeground
                    ? `hsl(${theme.colors.mutedForeground})`
                    : undefined,
                  fontFamily: theme.fonts?.body || 'inherit',
                }}
              >
                {theme.fonts?.heading?.split(',')[0] || 'Default'}
              </div>
            </div>
          </div>
        </div>
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
          <div
            key={theme.name}
            className={`cursor-pointer transition-all hover:opacity-90 ${
              currentTheme.name === theme.name ? 'ring-2' : 'hover:ring-1'
            }`}
            style={{
              backgroundColor: theme.backgroundImage
                ? `hsla(${theme.colors?.card || '0 0% 98%'}, ${theme.backgroundOpacity?.card ?? 0.75})`
                : theme.colors?.card
                  ? `hsl(${theme.colors.card})`
                  : undefined,
              color: theme.colors?.cardForeground
                ? `hsl(${theme.colors.cardForeground})`
                : undefined,
              border: theme.colors?.border
                ? `1px solid hsl(${theme.colors.border})`
                : undefined,
              borderRadius: theme.corners?.lg || '0.5rem',
              boxShadow: theme.shadows?.card || undefined,
              fontFamily: theme.fonts?.body || 'inherit',
              backgroundImage: theme.backgroundImage
                ? `url(${theme.backgroundImage})`
                : undefined,
              backgroundSize: theme.backgroundImage ? 'cover' : undefined,
              backgroundPosition: theme.backgroundImage ? 'center' : undefined,
              outline:
                currentTheme.name === theme.name
                  ? `2px solid ${currentTheme.colors?.primary ? `hsl(${currentTheme.colors.primary})` : 'currentColor'}`
                  : 'none',
            }}
            onClick={() => setTheme(theme.name)}
          >
            <div className='p-3'>
              <div className='flex items-center justify-between mb-2'>
                <h4
                  className='font-medium text-xs'
                  style={{ fontFamily: theme.fonts?.heading || 'inherit' }}
                >
                  {theme.displayName}
                </h4>
                {currentTheme.name === theme.name && (
                  <Check
                    className='h-3 w-3'
                    style={{
                      color: currentTheme.colors?.primary
                        ? `hsl(${currentTheme.colors.primary})`
                        : 'currentColor',
                    }}
                  />
                )}
              </div>

              <div className='flex gap-1'>
                <div
                  className='h-2 w-2'
                  style={{
                    backgroundColor: theme.colors?.primary
                      ? `hsl(${theme.colors.primary})`
                      : undefined,
                    borderRadius: theme.corners?.sm || '0.125rem',
                  }}
                />
                <div
                  className='h-2 w-2'
                  style={{
                    backgroundColor: theme.colors?.secondary
                      ? `hsl(${theme.colors.secondary})`
                      : undefined,
                    borderRadius: theme.corners?.sm || '0.125rem',
                  }}
                />
                <div
                  className='h-2 w-2'
                  style={{
                    backgroundColor: theme.colors?.accent
                      ? `hsl(${theme.colors.accent})`
                      : undefined,
                    borderRadius: theme.corners?.sm || '0.125rem',
                  }}
                />
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
