import {
  type Theme,
  getButtonStyles,
  getHeadingStyles,
  getMutedTextStyles,
  getThemeStyles,
} from '@/lib/theme';
import { Check } from 'lucide-react';

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
  const styles = getThemeStyles(theme);

  // Handle background image with transparency
  if (theme.backgroundImage) {
    const cardColor = theme.colors?.card || 'hsl(0 0% 100%)';
    const opacity = theme.backgroundOpacity?.card ?? 0.75;
    // Handle different color formats for transparency
    if (cardColor.startsWith('hsl')) {
      styles.backgroundColor = `hsla(${cardColor.slice(4, -1)}, ${opacity})`;
    } else if (cardColor.startsWith('rgb')) {
      styles.backgroundColor = `rgba(${cardColor.slice(4, -1)}, ${opacity})`;
    } else {
      // For other formats, use CSS color-mix as fallback
      styles.backgroundColor = `color-mix(in srgb, ${cardColor} ${opacity * 100}%, transparent)`;
    }
  }

  // Add fallbacks for missing properties
  if (!styles.backgroundColor && variant === 'default') {
    styles.backgroundColor = 'hsl(0 0% 100%)';
  }
  if (!styles.color && variant === 'default') {
    styles.color = 'hsl(0 0% 0%)';
  }
  if (!styles.border && variant === 'default') {
    styles.border = '1px solid hsl(0 0% 90%)';
  }
  if (!styles.borderRadius) {
    styles.borderRadius = '0.5rem';
  }
  if (!styles.boxShadow && variant === 'default') {
    styles.boxShadow = '0 1px 3px 0 rgb(0 0 0 / 0.1)';
  }
  if (!styles.fontFamily && variant === 'default') {
    styles.fontFamily = 'system-ui, sans-serif';
  }

  // Selection outline - use current theme's colors for selection indicator
  if (isSelected) {
    if (currentTheme.colors?.primary) {
      styles.outline = `2px solid ${currentTheme.colors.primary}`;
    } else {
      // Fallback that doesn't depend on ambient theme
      styles.outline = '2px solid hsl(220 13% 91%)';
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
    const buttonStyles = getButtonStyles(theme, 'default');
    const headingStyles = getHeadingStyles(theme);
    const mutedTextStyles = getMutedTextStyles(theme);

    // Add fallbacks for button styles
    if (!buttonStyles.backgroundColor) {
      buttonStyles.backgroundColor = 'hsl(220 13% 91%)'; // Default gray
    }
    if (!buttonStyles.color) {
      buttonStyles.color = 'hsl(0 0% 0%)'; // Default black
    }
    if (!buttonStyles.border) {
      buttonStyles.border = '1px solid hsl(0 0% 90%)';
    }
    if (!buttonStyles.borderRadius) {
      buttonStyles.borderRadius = '0.375rem';
    }
    if (!buttonStyles.boxShadow) {
      buttonStyles.boxShadow = '0 1px 2px 0 rgb(0 0 0 / 0.05)';
    }

    // Add fallbacks for heading styles
    if (!headingStyles.fontFamily) {
      headingStyles.fontFamily = 'system-ui, sans-serif';
    }

    // Add fallbacks for muted text styles
    if (!mutedTextStyles.color) {
      mutedTextStyles.color = 'hsl(0 0% 45%)'; // Default muted color
    }
    if (!mutedTextStyles.fontFamily) {
      mutedTextStyles.fontFamily = 'inherit';
    }

    const checkStyles: Record<string, string | undefined> = {};
    if (currentTheme.colors?.primary) {
      checkStyles.color = currentTheme.colors.primary;
    } else {
      checkStyles.color = 'hsl(220 13% 91%)'; // Default gray
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
    const headingStyles = getHeadingStyles(theme);

    // Add fallbacks for heading styles
    if (!headingStyles.fontFamily) {
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
