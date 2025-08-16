# Custom Theme System

This React Tauri app includes a comprehensive custom theme system that allows users to choose from multiple color themes. The system is built on top of Tailwind CSS and uses CSS custom properties (CSS variables) for consistent theming across all components.

## Features

- **Multiple Themes**: Pre-built themes including Light, Dark, Ocean Blue, Forest Green, Royal Purple, and Sunset Orange
- **Persistent Storage**: Theme preferences are saved to localStorage and persist across sessions
- **Real-time Switching**: Themes can be changed instantly without page reload
- **Component Integration**: All UI components automatically adapt to the selected theme
- **Extensible**: Easy to add new themes or modify existing ones

## How It Works

### Theme Structure

Each theme is defined as a TypeScript object with the following structure:

```typescript
interface Theme {
  name: string; // Unique identifier
  displayName: string; // User-friendly name
  colors: {
    background: string; // HSL color values (e.g., "0 0% 100%")
    foreground: string;
    primary: string;
    // ... other color properties
  };
}
```

### CSS Variables

The theme system uses CSS custom properties that are automatically applied to the document root:

```css
:root {
  --background: 0 0% 100%;
  --foreground: 0 0% 3.9%;
  --primary: 0 0% 9%;
  /* ... other variables */
}
```

### Tailwind Integration

Tailwind CSS classes automatically use these CSS variables:

```jsx
// These classes will use the current theme's colors
<div className="bg-background text-foreground">
<button className="bg-primary text-primary-foreground">
```

## Usage

### Basic Theme Usage

```tsx
import { useTheme } from '@/contexts/ThemeContext';

function MyComponent() {
  const { currentTheme, setTheme, availableThemes } = useTheme();

  return (
    <div>
      <h1>Current theme: {currentTheme.displayName}</h1>
      <button onClick={() => setTheme('blue')}>Switch to Ocean Blue</button>
    </div>
  );
}
```

### Using Theme Colors in Styled Components

```tsx
import { useThemeColors } from '@/hooks/useThemeColors';

function CustomComponent() {
  const colors = useThemeColors();

  return (
    <div
      style={{
        backgroundColor: colors.primary,
        color: colors.primaryForeground,
      }}
    >
      Custom styled content
    </div>
  );
}
```

### Theme Selector Components

```tsx
import { ThemeSelector, ThemeSelectorCompact } from '@/components/ThemeSelector';

// Full theme selector with previews
<ThemeSelector />

// Compact theme selector for quick switching
<ThemeSelectorCompact />
```

## Adding New Themes

### 1. Define the Theme

Add a new theme to the `themes` array in `src/lib/themes.ts`:

```typescript
{
  name: 'midnight',
  displayName: 'Midnight',
  colors: {
    background: '240 10% 3.9%',
    foreground: '0 0% 98%',
    primary: '240 5.9% 10%',
    primaryForeground: '0 0% 98%',
    secondary: '240 4.8% 95.9%',
    secondaryForeground: '240 5.9% 10%',
    // ... define all other colors
  },
}
```

### 2. Color Guidelines

When creating new themes, follow these guidelines:

- **Background**: Should provide good contrast with foreground text
- **Primary**: Main brand/accent color for buttons and important elements
- **Secondary**: Subtle background color for cards and secondary elements
- **Muted**: Very subtle background for disabled or less important elements
- **Destructive**: Red-based color for error states and destructive actions

### 3. HSL Color Format

All colors should be in HSL format: `"hue saturation% lightness%"`

- **Hue**: 0-360 (color wheel position)
- **Saturation**: 0-100% (color intensity)
- **Lightness**: 0-100% (brightness)

Example: `"220 70% 50%"` = blue with 70% saturation and 50% lightness

## Theme Context

The `ThemeProvider` wraps the app and provides theme functionality:

```tsx
import { ThemeProvider } from '@/contexts/ThemeContext';

function App() {
  return <ThemeProvider>{/* Your app components */}</ThemeProvider>;
}
```

## Available Hooks

### useTheme()

Provides access to theme state and functions:

```tsx
const { currentTheme, setTheme, availableThemes } = useTheme();
```

### useThemeColors()

Provides theme colors as CSS color strings:

```tsx
const colors = useThemeColors();
// colors.primary = "hsl(220 70% 50%)"
```

## Best Practices

1. **Use Tailwind Classes**: Prefer Tailwind classes over inline styles for theme consistency
2. **Test Contrast**: Ensure sufficient contrast between text and background colors
3. **Consistent Naming**: Use the established color naming convention
4. **Accessibility**: Consider colorblind users when choosing theme colors
5. **Performance**: Theme switching is optimized and doesn't cause re-renders

## Testing Themes

Use the theme demo page at `/theme-demo` to preview all themes and see how they look with different UI components.

## Migration from Dark Mode

The existing dark mode system is still supported for backward compatibility. The new theme system extends this functionality by providing more theme options while maintaining the same CSS variable structure.

## Troubleshooting

### Theme Not Applying

1. Ensure `ThemeProvider` wraps your app
2. Check that CSS variables are being set correctly
3. Verify Tailwind classes are using the correct color names

### Colors Not Updating

1. Check browser developer tools for CSS variable values
2. Ensure the theme object has all required color properties
3. Verify the `applyTheme` function is being called

### Performance Issues

1. Theme switching is optimized and should be instant
2. Avoid creating new theme objects on every render
3. Use the provided hooks instead of accessing theme context directly
