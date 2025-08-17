import { Check } from 'lucide-react';
import { type Theme } from '@/lib/theme';

interface ThemeCardProps {
  theme: Theme;
  isSelected: boolean;
  onSelect: (themeName: string) => void;
  variant?: 'default' | 'compact' | 'simple';
  className?: string;
}

export function ThemeCard({
  theme,
  isSelected,
  onSelect,
  variant = 'default',
  className = '',
}: ThemeCardProps) {
  const baseStyles = {
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
    outline: isSelected
      ? `2px solid ${theme.colors?.primary ? `hsl(${theme.colors.primary})` : 'hsl(var(--primary))'}`
      : 'none',
  };

  const simpleStyles = {
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
    outline: isSelected
      ? `2px solid ${theme.colors?.primary ? `hsl(${theme.colors.primary})` : 'currentColor'}`
      : 'none',
  };

  const styles = variant === 'simple' ? simpleStyles : baseStyles;

  const renderDefaultContent = () => (
    <div className='p-4'>
      <div className='flex items-center justify-between mb-3'>
        <h3
          className='font-medium text-sm'
          style={{ fontFamily: theme.fonts?.heading || 'var(--font-heading)' }}
        >
          {theme.displayName}
        </h3>
        {isSelected && (
          <Check
            className='h-4 w-4'
            style={{
              color: theme.colors?.primary
                ? `hsl(${theme.colors.primary})`
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
  );

  const renderSimpleContent = () => (
    <div className='p-3'>
      <div className='flex items-center justify-between mb-2'>
        <h4
          className='font-medium text-xs'
          style={{ fontFamily: theme.fonts?.heading || 'inherit' }}
        >
          {theme.displayName}
        </h4>
        {isSelected && (
          <Check
            className='h-3 w-3'
            style={{
              color: theme.colors?.primary
                ? `hsl(${theme.colors.primary})`
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
  );

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
