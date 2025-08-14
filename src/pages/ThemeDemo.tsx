import React from 'react';
import { useTheme } from '@/contexts/ThemeContext';
import { ThemeSelector, ThemeSelectorCompact } from '@/components/ThemeSelector';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Info, Palette, Sparkles } from 'lucide-react';
import Header from '@/components/Header';
import Layout from '@/components/Layout';

export default function ThemeDemo() {
  console.log('ThemeDemo component rendering...');
  
  try {
    const { currentTheme } = useTheme();
    console.log('Current theme:', currentTheme);

    return (
      <Layout>
        <Header title="Theme Demo" back={() => window.history.back()} />
      
      <div className="container mx-auto p-6 space-y-8">
        {/* Theme Selector */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Palette className="h-5 w-5" />
              Choose Your Theme
            </CardTitle>
            <CardDescription>
              Select from our collection of beautiful themes
            </CardDescription>
          </CardHeader>
          <CardContent>
            <ThemeSelector />
          </CardContent>
        </Card>

        {/* Current Theme Info */}
        <Card>
          <CardHeader>
            <CardTitle>Current Theme: {currentTheme.displayName}</CardTitle>
            <CardDescription>
              This is how your selected theme looks across different components
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div className="space-y-2">
                <Label>Primary</Label>
                <div 
                  className="h-12 rounded-md border"
                  style={{ backgroundColor: `hsl(${currentTheme.colors.primary})` }}
                />
              </div>
              <div className="space-y-2">
                <Label>Secondary</Label>
                <div 
                  className="h-12 rounded-md border"
                  style={{ backgroundColor: `hsl(${currentTheme.colors.secondary})` }}
                />
              </div>
              <div className="space-y-2">
                <Label>Accent</Label>
                <div 
                  className="h-12 rounded-md border"
                  style={{ backgroundColor: `hsl(${currentTheme.colors.accent})` }}
                />
              </div>
              <div className="space-y-2">
                <Label>Destructive</Label>
                <div 
                  className="h-12 rounded-md border"
                  style={{ backgroundColor: `hsl(${currentTheme.colors.destructive})` }}
                />
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Component Examples */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {/* Buttons */}
          <Card>
            <CardHeader>
              <CardTitle>Buttons</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex flex-wrap gap-2">
                <Button>Default</Button>
                <Button variant="secondary">Secondary</Button>
                <Button variant="outline">Outline</Button>
                <Button variant="ghost">Ghost</Button>
                <Button variant="destructive">Destructive</Button>
              </div>
              <div className="flex flex-wrap gap-2">
                <Button size="sm">Small</Button>
                <Button size="lg">Large</Button>
              </div>
            </CardContent>
          </Card>

          {/* Form Elements */}
          <Card>
            <CardHeader>
              <CardTitle>Form Elements</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="email">Email</Label>
                <Input id="email" placeholder="Enter your email" />
              </div>
              <div className="flex items-center space-x-2">
                <Switch id="notifications" />
                <Label htmlFor="notifications">Enable notifications</Label>
              </div>
              <div className="w-full bg-secondary rounded-full h-2">
                <div 
                  className="bg-primary h-2 rounded-full transition-all duration-300" 
                  style={{ width: '65%' }}
                />
              </div>
            </CardContent>
          </Card>

          {/* Badges and Alerts */}
          <Card>
            <CardHeader>
              <CardTitle>Badges & Alerts</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex flex-wrap gap-2">
                <Badge>Default</Badge>
                <Badge variant="secondary">Secondary</Badge>
                <Badge variant="outline">Outline</Badge>
              </div>
              <Alert>
                <Info className="h-4 w-4" />
                <AlertDescription>
                  This is an informational alert that adapts to your theme.
                </AlertDescription>
              </Alert>
            </CardContent>
          </Card>

          {/* Tabs */}
          <Card>
            <CardHeader>
              <CardTitle>Tabs</CardTitle>
            </CardHeader>
            <CardContent>
              <Tabs defaultValue="account" className="w-full">
                <TabsList className="grid w-full grid-cols-2">
                  <TabsTrigger value="account">Account</TabsTrigger>
                  <TabsTrigger value="password">Password</TabsTrigger>
                </TabsList>
                <TabsContent value="account" className="mt-4">
                  <p className="text-sm text-muted-foreground">
                    Account settings content goes here.
                  </p>
                </TabsContent>
                <TabsContent value="password" className="mt-4">
                  <p className="text-sm text-muted-foreground">
                    Password settings content goes here.
                  </p>
                </TabsContent>
              </Tabs>
            </CardContent>
          </Card>
        </div>

        {/* Compact Theme Selector */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Sparkles className="h-5 w-5" />
              Quick Theme Switcher
            </CardTitle>
            <CardDescription>
              Compact theme selector for quick switching
            </CardDescription>
          </CardHeader>
          <CardContent>
            <ThemeSelectorCompact />
          </CardContent>
        </Card>

        <Separator />

        {/* Theme Information */}
        <Card>
          <CardHeader>
            <CardTitle>About Themes</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <p className="text-sm text-muted-foreground">
              Our theme system uses CSS custom properties (CSS variables) to provide 
              consistent theming across all components. Each theme defines a complete 
              color palette that includes:
            </p>
            <ul className="text-sm text-muted-foreground space-y-1 ml-4">
              <li>• Background and foreground colors</li>
              <li>• Primary, secondary, and accent colors</li>
              <li>• Muted and destructive colors</li>
              <li>• Border and input colors</li>
              <li>• Chart colors for data visualization</li>
            </ul>
            <p className="text-sm text-muted-foreground">
              Themes are automatically saved to localStorage and will persist across 
              browser sessions. You can easily add new themes by extending the themes 
              array in the theme configuration.
            </p>
          </CardContent>
        </Card>
      </div>
    </Layout>
    );
  } catch (error) {
    console.error('Error in ThemeDemo:', error);
    return (
      <div className="p-6">
        <h1>Theme Demo</h1>
        <p>Error loading theme demo: {error instanceof Error ? error.message : 'Unknown error'}</p>
      </div>
    );
  }
}
