import { useThemeColors } from '@/hooks/useThemeColors';
import { useThemeFonts } from '@/hooks/useThemeFonts';

/**
 * Example component demonstrating how to use theme fonts
 */
export function FontExample() {
  const fonts = useThemeFonts();
  const { fonts: fontFamilies } = useThemeColors();

  return (
    <div className='space-y-6 p-6'>
      <h2 className='text-2xl font-bold'>Font Usage Examples</h2>

      {/* Method 1: Using the useThemeFonts hook with inline styles */}
      <div className='space-y-4'>
        <h3 className='text-lg font-semibold'>Method 1: Inline Styles</h3>
        <div className='space-y-2'>
          <p style={fonts.styles.heading} className='text-xl font-bold'>
            Heading text using theme heading font
          </p>
          <p style={fonts.styles.body}>
            Body text using theme body font. This is great for custom
            components.
          </p>
          <code
            style={fonts.styles.mono}
            className='block bg-muted p-2 rounded'
          >
            const code = 'monospace font example';
          </code>
        </div>
      </div>

      {/* Method 2: Using CSS variables */}
      <div className='space-y-4'>
        <h3 className='text-lg font-semibold'>Method 2: CSS Variables</h3>
        <div className='space-y-2'>
          <p style={{ fontFamily: fonts.cssVars.serif }} className='text-lg'>
            Serif font using CSS variable
          </p>
          <p style={{ fontFamily: fonts.cssVars.sans }}>
            Sans-serif font using CSS variable
          </p>
        </div>
      </div>

      {/* Method 3: Direct font family strings */}
      <div className='space-y-4'>
        <h3 className='text-lg font-semibold'>
          Method 3: Direct Font Families
        </h3>
        <div className='space-y-2'>
          <p style={{ fontFamily: fontFamilies.heading }}>
            Using fontFamilies.heading from useThemeColors
          </p>
          <p style={{ fontFamily: fonts.mono }}>
            Using fonts.mono directly: {fonts.mono}
          </p>
        </div>
      </div>

      {/* Method 4: Combining with Tailwind classes */}
      <div className='space-y-4'>
        <h3 className='text-lg font-semibold'>
          Method 4: With Tailwind Classes
        </h3>
        <div className='space-y-2'>
          <p
            style={fonts.styles.heading}
            className='text-2xl font-bold text-primary'
          >
            Styled heading with Tailwind + theme font
          </p>
          <p
            style={fonts.styles.body}
            className='text-sm text-muted-foreground'
          >
            Body text with Tailwind styling and theme font
          </p>
        </div>
      </div>
    </div>
  );
}
