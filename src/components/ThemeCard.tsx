import { Check } from 'lucide-react';
import { type Theme } from '@/lib/theme';

interface ThemeCardProps {
  theme: Theme;
  currentTheme: Theme;
  isSelected: boolean;
  onSelect: (themeName: string) => void;
  variant?: 'default' | 'compact' | 'simple';
  className?: string;
}

// Helper function to create theme styles that properly handle missing properties
function createThemeStyles(
  theme: Theme,
  currentTheme: Theme,
  isSelected: boolean,
  variant: 'default' | 'compact' | 'simple' = 'default',
) {
  const styles: Record<string, string | undefined> = {};

  // Background color - only use theme's own colors, never CSS variables
  if (theme.backgroundImage) {
    styles.backgroundColor = theme.colors?.card || '#ffffff';
  } else if (theme.colors?.card) {
    styles.backgroundColor = theme.colors.card;
  } else {
    // Default fallback that doesn't depend on ambient theme
    styles.backgroundColor =
      variant === 'default' ? '#ffffff' : undefined;
  }

  // Text color - only use theme's own colors
  if (theme.colors?.cardForeground) {
    styles.color = theme.colors.cardForeground;
  } else {
    // Default fallback that doesn't depend on ambient theme
    styles.color = variant === 'default' ? '#000000' : undefined;
  }

  // Border - only use theme's own colors
  if (theme.colors?.border) {
    styles.border = `1px solid ${theme.colors.border}`;
  } else {
    // Default fallback that doesn't depend on ambient theme
    styles.border =
      variant === 'default' ? '1px solid #e5e5e5' : undefined;
  }

  // Border radius - use theme's own values or fixed defaults
  if (theme.corners?.lg) {
    styles.borderRadius = theme.corners.lg;
  } else {
    styles.borderRadius = variant === 'default' ? '0.5rem' : '0.5rem';
  }

  // Box shadow - use theme's own values or fixed defaults
  if (theme.shadows?.card) {
    styles.boxShadow = theme.shadows.card;
  } else {
    styles.boxShadow =
      variant === 'default' ? '0 1px 3px 0 rgb(0 0 0 / 0.1)' : undefined;
  }

  // Font family - use theme's own values or fixed defaults
  if (theme.fonts?.body) {
    styles.fontFamily = theme.fonts.body;
  } else {
    styles.fontFamily =
      variant === 'default' ? 'system-ui, sans-serif' : 'inherit';
  }

  // Background image
  if (theme.backgroundImage) {
    styles.backgroundImage = `url(${theme.backgroundImage})`;
    styles.backgroundSize = 'cover';
    styles.backgroundPosition = 'center';
  }

  // Selection outline - use current theme's colors for selection indicator
  if (isSelected) {
    if (currentTheme.colors?.primary) {
      styles.outline = `2px solid ${currentTheme.colors.primary}`;
    } else {
      // Fallback that doesn't depend on ambient theme
      styles.outline = '2px solid #e5e7eb';
    }
  } else {
    styles.outline = 'none';
  }

  return styles;
}

export function ThemeCard({
  theme,
  currentTheme,
  isSelected,
  onSelect,
  variant = 'default',
  className = '',
}: ThemeCardProps) {
  const styles = createThemeStyles(theme, currentTheme, isSelected, variant);

  const renderDefaultContent = () => {
    const buttonStyles: Record<string, string | undefined> = {};
    if (theme.colors?.primary) {
      buttonStyles.backgroundColor = theme.colors.primary;
    } else {
      buttonStyles.backgroundColor = '#e5e7eb'; // Default gray
    }
    if (theme.colors?.border) {
      buttonStyles.borderColor = theme.colors.border;
      buttonStyles.border = `1px solid ${theme.colors.border}`;
    } else {
      buttonStyles.borderColor = '#e5e5e5';
      buttonStyles.border = '1px solid #e5e5e5';
    }
    if (theme.colors?.primaryForeground) {
      buttonStyles.color = theme.colors.primaryForeground;
    } else {
      buttonStyles.color = '#000000'; // Default black
    }
    if (theme.fonts?.heading) {
      buttonStyles.fontFamily = theme.fonts.heading;
    } else {
      buttonStyles.fontFamily = 'system-ui, sans-serif';
    }
    if (theme.corners?.md) {
      buttonStyles.borderRadius = theme.corners.md;
    } else {
      buttonStyles.borderRadius = '0.375rem';
    }
    if (theme.shadows?.button) {
      buttonStyles.boxShadow = theme.shadows.button;
    } else {
      buttonStyles.boxShadow = '0 1px 2px 0 rgb(0 0 0 / 0.05)';
    }

    const headingStyles: Record<string, string | undefined> = {};
    if (theme.fonts?.heading) {
      headingStyles.fontFamily = theme.fonts.heading;
    } else {
      headingStyles.fontFamily = 'system-ui, sans-serif';
    }

    const checkStyles: Record<string, string | undefined> = {};
    if (currentTheme.colors?.primary) {
      checkStyles.color = currentTheme.colors.primary;
    } else {
      checkStyles.color = '#e5e7eb'; // Default gray
    }

    const mutedTextStyles: Record<string, string | undefined> = {};
    if (theme.colors?.mutedForeground) {
      mutedTextStyles.color = theme.colors.mutedForeground;
    } else {
      mutedTextStyles.color = '#737373'; // Default muted color
    }
    if (theme.fonts?.body) {
      mutedTextStyles.fontFamily = theme.fonts.body;
    } else {
      mutedTextStyles.fontFamily = 'inherit';
    }

    return (
      <div className='p-4'>
        <div className='flex items-center justify-between mb-3'>
          <h3 className='font-medium text-sm' style={headingStyles}>
            {theme.displayName}
          </h3>
          {isSelected && <Check className='h-4 w-4' style={checkStyles} />}
        </div>

        {/* Theme preview */}
        <div className='space-y-2'>
          <div className='h-8 flex items-center px-2' style={buttonStyles}>
            <span className='text-xs font-medium'>Aa</span>
          </div>
          <div className='flex gap-1'>
            <div
              className='h-4 w-4'
              style={{
                backgroundColor: theme.colors?.primary || undefined,
                borderRadius: theme.corners?.sm || '0.125rem',
              }}
            />
            <div
              className='h-4 w-4'
              style={{
                backgroundColor: theme.colors?.secondary || undefined,
                borderRadius: theme.corners?.sm || '0.125rem',
              }}
            />
            <div
              className='h-4 w-4'
              style={{
                backgroundColor: theme.colors?.accent || undefined,
                borderRadius: theme.corners?.sm || '0.125rem',
              }}
            />
            <div
              className='h-4 w-4'
              style={{
                backgroundColor: theme.colors?.destructive || undefined,
                borderRadius: theme.corners?.sm || '0.125rem',
              }}
            />
          </div>
          <div className='text-xs truncate' style={mutedTextStyles}>
            {theme.fonts?.heading?.split(',')[0] || 'Default'}
          </div>
        </div>
      </div>
    );
  };

  const renderSimpleContent = () => {
    const headingStyles: Record<string, string | undefined> = {};
    if (theme.fonts?.heading) {
      headingStyles.fontFamily = theme.fonts.heading;
    } else {
      headingStyles.fontFamily = 'inherit';
    }

    const checkStyles: Record<string, string | undefined> = {};
    if (currentTheme.colors?.primary) {
      checkStyles.color = currentTheme.colors.primary;
    } else {
      checkStyles.color = 'currentColor';
    }

    return (
      <div className='p-3'>
        <div className='flex items-center justify-between mb-2'>
          <h4 className='font-medium text-xs' style={headingStyles}>
            {theme.displayName}
          </h4>
          {isSelected && <Check className='h-3 w-3' style={checkStyles} />}
        </div>

        <div className='flex gap-1'>
          <div
            className='h-2 w-2'
            style={{
              backgroundColor: theme.colors?.primary || undefined,
              borderRadius: theme.corners?.sm || '0.125rem',
            }}
          />
          <div
            className='h-2 w-2'
            style={{
              backgroundColor: theme.colors?.secondary || undefined,
              borderRadius: theme.corners?.sm || '0.125rem',
            }}
          />
          <div
            className='h-2 w-2'
            style={{
              backgroundColor: theme.colors?.accent || undefined,
              borderRadius: theme.corners?.sm || '0.125rem',
            }}
          />
        </div>
      </div>
    );
  };

  return (
    <div
      className={`cursor-pointer transition-all hover:opacity-90 ${
        isSelected ? 'ring-2' : 'hover:ring-1'
      } ${className}`}
      style={styles}
      onClick={() => onSelect(theme.name)}
    >
      {variant === 'simple' ? renderSimpleContent() : renderDefaultContent()}
    </div>
  );
}
