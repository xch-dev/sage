import { commands } from '@/bindings';
import { useTheme } from '@/contexts/ThemeContext';
import { useErrors } from '@/hooks/useErrors';
import {
  type Theme,
  applyTheme,
  getPreviewButtonStyles,
  getPreviewHeadingStyles,
  getPreviewMutedTextStyles,
  getPreviewTextStyles,
} from '@/lib/theme';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { Check, Trash2 } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';
import { toast } from 'react-toastify';
import { Button } from './ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from './ui/dialog';

interface ThemeCardProps {
  theme: Theme;
  currentTheme: Theme;
  isSelected: boolean;
  onSelect: (themeName: string) => void;
  variant?: 'default' | 'compact' | 'simple';
  className?: string;
}

export function ThemeCard({
  theme,
  currentTheme,
  isSelected,
  onSelect,
  variant = 'default',
  className = '',
}: ThemeCardProps) {
  const cardRef = useRef<HTMLDivElement>(null);
  const { reloadThemes } = useTheme();
  const { addError } = useErrors();
  const [isDeleting, setIsDeleting] = useState(false);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);

  const handleDeleteClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setShowDeleteConfirm(true);
  };

  const handleDeleteTheme = async () => {
    if (theme.isUserTheme) {
      setIsDeleting(true);
      try {
        await commands.deleteUserTheme({ nft_id: theme.name });
        await reloadThemes();
        toast.success(t`Theme deleted successfully`);
        setShowDeleteConfirm(false);
      } catch (error) {
        addError({
          kind: 'internal',
          reason:
            error instanceof Error ? error.message : 'Unknown error occurred',
        });
      } finally {
        setIsDeleting(false);
      }
    }
  };

  useEffect(() => {
    if (cardRef.current) {
      // Apply the theme to this specific element as a preview
      applyTheme(theme, cardRef.current, true);

      // Explicitly set the background color to ensure isolation from ambient theme
      const cardColor =
        theme.colors?.card || theme.colors?.background || 'hsl(0 0% 100%)';
      cardRef.current.style.backgroundColor = cardColor;
    }
  }, [theme]);

  // Only apply selection outline as inline style
  const selectionStyle = isSelected
    ? {
        outline: `2px solid ${currentTheme.colors?.primary || 'hsl(220 13% 91%)'}`,
      }
    : {};

  const renderDefaultContent = () => {
    const buttonStyles = getPreviewButtonStyles(theme, 'default');
    const headingStyles = getPreviewHeadingStyles(theme);
    const mutedTextStyles = getPreviewMutedTextStyles(theme);
    const textStyles = getPreviewTextStyles(theme);

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

    // Add fallbacks for text styles
    if (!textStyles.color) {
      textStyles.color = 'hsl(0 0% 0%)'; // Default black
    }
    if (!textStyles.fontFamily) {
      textStyles.fontFamily = 'inherit';
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
          <div className='flex items-center gap-2'>
            {isSelected && <Check className='h-4 w-4' style={checkStyles} />}
            {theme.isUserTheme && (
              <Button
                onClick={handleDeleteClick}
                disabled={isDeleting}
                variant='ghost'
                size='icon'
                aria-label={t`Delete theme ${theme.displayName}`}
                title={t`Delete theme ${theme.displayName}`}
              >
                <Trash2
                  className='h-4 w-4 text-destructive'
                  aria-hidden='true'
                />
              </Button>
            )}
          </div>
        </div>

        {/* Theme preview */}
        <div className='space-y-2'>
          <div className='h-8 flex items-center px-2' style={buttonStyles}>
            <span className='text-xs font-medium' style={textStyles}>
              Aa
            </span>
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
    const headingStyles = getPreviewHeadingStyles(theme);

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
          <div className='flex items-center gap-1'>
            {isSelected && (
              <Check
                className='h-3 w-3'
                style={checkStyles}
                aria-label={t`Theme selected`}
              />
            )}
            {theme.isUserTheme && (
              <Button
                onClick={handleDeleteClick}
                disabled={isDeleting}
                variant='ghost'
                size='icon'
                aria-label={t`Delete theme ${theme.displayName}`}
                title={t`Delete theme ${theme.displayName}`}
              >
                <Trash2
                  className='h-4 w-4 text-destructive'
                  aria-hidden='true'
                />
              </Button>
            )}
          </div>
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
    <>
      <div
        ref={cardRef}
        className={`cursor-pointer transition-all hover:opacity-90 text-card-foreground border border-border rounded-lg shadow-card theme-card-isolated ${
          isSelected ? 'ring-2' : 'hover:ring-1'
        } ${className}`}
        style={selectionStyle}
        onClick={() => onSelect(theme.name)}
      >
        {variant === 'simple' ? renderSimpleContent() : renderDefaultContent()}
      </div>

      <Dialog open={showDeleteConfirm} onOpenChange={setShowDeleteConfirm}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Trans>Delete Theme</Trans>
            </DialogTitle>
            <DialogDescription>
              <Trans>
                Are you sure you want to delete the theme &quot;
                {theme.displayName}&quot;?
              </Trans>
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              variant='outline'
              onClick={() => setShowDeleteConfirm(false)}
            >
              <Trans>Cancel</Trans>
            </Button>
            <Button
              variant='destructive'
              onClick={handleDeleteTheme}
              disabled={isDeleting}
            >
              {isDeleting ? <Trans>Deleting...</Trans> : <Trans>Delete</Trans>}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
