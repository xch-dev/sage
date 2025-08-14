import React from 'react';
import { useTheme } from '@/contexts/ThemeContext';
import { Card, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Check } from 'lucide-react';

export function ThemeSelector() {
  const { currentTheme, setTheme, availableThemes } = useTheme();

  return (
    <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
      {availableThemes.map((theme) => (
        <Card
          key={theme.name}
          className={`cursor-pointer transition-all hover:shadow-md ${
            currentTheme.name === theme.name
              ? 'ring-2 ring-primary'
              : 'hover:ring-1 hover:ring-border'
          }`}
          onClick={() => setTheme(theme.name)}
        >
          <CardContent className="p-4">
            <div className="flex items-center justify-between mb-3">
              <h3 className="font-medium text-sm">{theme.displayName}</h3>
              {currentTheme.name === theme.name && (
                <Check className="h-4 w-4 text-primary" />
              )}
            </div>
            
            {/* Theme preview */}
            <div className="space-y-2">
              <div 
                className="h-8 rounded-md border"
                style={{ 
                  backgroundColor: `hsl(${theme.colors.primary})`,
                  borderColor: `hsl(${theme.colors.border})`
                }}
              />
              <div className="flex gap-1">
                <div 
                  className="h-4 w-4 rounded-sm"
                  style={{ backgroundColor: `hsl(${theme.colors.primary})` }}
                />
                <div 
                  className="h-4 w-4 rounded-sm"
                  style={{ backgroundColor: `hsl(${theme.colors.secondary})` }}
                />
                <div 
                  className="h-4 w-4 rounded-sm"
                  style={{ backgroundColor: `hsl(${theme.colors.accent})` }}
                />
                <div 
                  className="h-4 w-4 rounded-sm"
                  style={{ backgroundColor: `hsl(${theme.colors.destructive})` }}
                />
              </div>
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}

export function ThemeSelectorCompact() {
  const { currentTheme, setTheme, availableThemes } = useTheme();

  return (
    <div className="flex flex-wrap gap-2">
      {availableThemes.map((theme) => (
        <Button
          key={theme.name}
          variant={currentTheme.name === theme.name ? 'default' : 'outline'}
          size="sm"
          onClick={() => setTheme(theme.name)}
          className="flex items-center gap-2"
        >
          <div 
            className="w-3 h-3 rounded-full"
            style={{ backgroundColor: `hsl(${theme.colors.primary})` }}
          />
          {theme.displayName}
        </Button>
      ))}
    </div>
  );
}
