import Header from '@/components/Header';
import Layout from '@/components/Layout';
import { ThemeSelector } from '@/components/ThemeSelector';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { useTheme } from '@/contexts/ThemeContext';
import { Trans } from '@lingui/react/macro';
import { Info, Palette } from 'lucide-react';

export default function Themes() {
  const { currentTheme } = useTheme();
  try {
    console.log('Current theme:', currentTheme);

    return (
      <Layout>
        <Header title='Theme' back={() => window.history.back()} />

        <div className='flex-1 overflow-auto'>
          <div className='container mx-auto p-6 space-y-8'>
            {/* Theme Selector */}
            <Card>
              <CardHeader>
                <CardTitle className='flex items-center gap-2'>
                  <Palette className='h-5 w-5' />
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
                  This is how your selected theme looks across different
                  components
                </CardDescription>
              </CardHeader>
              <CardContent className='space-y-6'>
                {/* Color Palette */}
                <div>
                  <Label className='text-base font-semibold mb-3 block'>
                    Colors
                  </Label>
                  <div className='grid grid-cols-2 md:grid-cols-4 gap-4'>
                    <div className='space-y-2'>
                      <Label>Primary</Label>
                      <div
                        className='h-12 rounded-md border'
                        style={{
                          backgroundColor: `hsl(${currentTheme.colors.primary})`,
                        }}
                      />
                    </div>
                    <div className='space-y-2'>
                      <Label>Secondary</Label>
                      <div
                        className='h-12 rounded-md border'
                        style={{
                          backgroundColor: `hsl(${currentTheme.colors.secondary})`,
                        }}
                      />
                    </div>
                    <div className='space-y-2'>
                      <Label>Accent</Label>
                      <div
                        className='h-12 rounded-md border'
                        style={{
                          backgroundColor: `hsl(${currentTheme.colors.accent})`,
                        }}
                      />
                    </div>
                    <div className='space-y-2'>
                      <Label>Destructive</Label>
                      <div
                        className='h-12 rounded-md border'
                        style={{
                          backgroundColor: `hsl(${currentTheme.colors.destructive})`,
                        }}
                      />
                    </div>
                  </div>
                </div>

                {/* Font Examples */}
                <div>
                  <Label className='text-base font-semibold mb-3 block'>
                    Typography
                  </Label>
                  <div className='space-y-4'>
                    <div className='space-y-2'>
                      <Label>
                        <Trans>Heading Font</Trans>:{' '}
                        {currentTheme.fonts.heading}
                      </Label>
                      <p
                        className='text-2xl font-bold'
                        style={{ fontFamily: currentTheme.fonts.heading }}
                      >
                        <Trans>
                          The quick brown fox jumps over the lazy dog
                        </Trans>
                      </p>
                    </div>
                    <div className='space-y-2'>
                      <Label>
                        <Trans>Body Font</Trans>: {currentTheme.fonts.body}
                      </Label>
                      <p
                        className='text-base'
                        style={{ fontFamily: currentTheme.fonts.body }}
                      >
                        <Trans>
                          The quick brown fox jumps over the lazy dog. This is
                          regular body text that demonstrates how readable
                          content appears with the selected theme font.
                        </Trans>
                      </p>
                    </div>
                  </div>
                </div>

                {/* Corners & Shadows Examples */}
                <div>
                  <Label className='text-base font-semibold mb-3 block'>
                    Corners & Shadows
                  </Label>
                  <div className='space-y-4'>
                    <div className='space-y-2'>
                      <Label>
                        <Trans>Border Radius</Trans>: {currentTheme.corners.lg}
                      </Label>
                      <div className='flex gap-3 items-center'>
                        <div
                          className='w-12 h-12 bg-primary'
                          style={{ borderRadius: currentTheme.corners.none }}
                        />
                        <div
                          className='w-12 h-12 bg-primary'
                          style={{ borderRadius: currentTheme.corners.sm }}
                        />
                        <div
                          className='w-12 h-12 bg-primary'
                          style={{ borderRadius: currentTheme.corners.md }}
                        />
                        <div
                          className='w-12 h-12 bg-primary'
                          style={{ borderRadius: currentTheme.corners.lg }}
                        />
                        <div
                          className='w-12 h-12 bg-primary'
                          style={{ borderRadius: currentTheme.corners.xl }}
                        />
                      </div>
                    </div>
                    <div className='space-y-2'>
                      <Label>Card Shadow Style</Label>
                      <div
                        className='p-4 bg-card text-card-foreground border'
                        style={{
                          borderRadius: currentTheme.corners.lg,
                          boxShadow: currentTheme.shadows.card,
                        }}
                      >
                        <Trans>
                          This card shows the theme&apos;s shadow and corner
                          style.
                        </Trans>
                      </div>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* Component Examples */}
            <div className='grid grid-cols-1 md:grid-cols-2 gap-6'>
              {/* Buttons */}
              <Card>
                <CardHeader>
                  <CardTitle>Buttons</CardTitle>
                </CardHeader>
                <CardContent className='space-y-4'>
                  <div className='flex flex-wrap gap-2'>
                    <Button>
                      <Trans>Default</Trans>
                    </Button>
                    <Button variant='secondary'>
                      <Trans>Secondary</Trans>
                    </Button>
                    <Button variant='outline'>
                      <Trans>Outline</Trans>
                    </Button>
                    <Button variant='ghost'>
                      <Trans>Ghost</Trans>
                    </Button>
                    <Button variant='destructive'>
                      <Trans>Destructive</Trans>
                    </Button>
                  </div>
                  <div className='flex flex-wrap gap-2'>
                    <Button size='sm'>
                      <Trans>Small</Trans>
                    </Button>
                    <Button size='lg'>
                      <Trans>Large</Trans>
                    </Button>
                  </div>
                </CardContent>
              </Card>

              {/* Form Elements */}
              <Card>
                <CardHeader>
                  <CardTitle>Form Elements</CardTitle>
                </CardHeader>
                <CardContent className='space-y-4'>
                  <div className='space-y-2'>
                    <Label htmlFor='email'>
                      <Trans>Email</Trans>
                    </Label>
                    <Input id='email' placeholder='Enter your email' />
                  </div>
                  <div className='flex items-center space-x-2'>
                    <Switch id='notifications' />
                    <Label htmlFor='notifications'>
                      <Trans>Enable notifications</Trans>
                    </Label>
                  </div>
                  <div className='w-full bg-secondary rounded-full h-2'>
                    <div
                      className='bg-primary h-2 rounded-full transition-all duration-300'
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
                <CardContent className='space-y-4'>
                  <div className='flex flex-wrap gap-2'>
                    <Badge>
                      <Trans>Default</Trans>
                    </Badge>
                    <Badge variant='secondary'>
                      <Trans>Secondary</Trans>
                    </Badge>
                    <Badge variant='outline'>
                      <Trans>Outline</Trans>
                    </Badge>
                  </div>
                  <Alert>
                    <Info className='h-4 w-4' />
                    <AlertDescription>
                      <Trans>
                        This is an informational alert that adapts to your
                        theme.
                      </Trans>
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
                  <Tabs defaultValue='account' className='w-full'>
                    <TabsList className='grid w-full grid-cols-2'>
                      <TabsTrigger value='account'>
                        <Trans>Account</Trans>
                      </TabsTrigger>
                      <TabsTrigger value='password'>
                        <Trans>Password</Trans>
                      </TabsTrigger>
                    </TabsList>
                    <TabsContent value='account' className='mt-4'>
                      <p className='text-sm text-muted-foreground'>
                        <Trans>Account settings content goes here.</Trans>
                      </p>
                    </TabsContent>
                    <TabsContent value='password' className='mt-4'>
                      <p className='text-sm text-muted-foreground'>
                        <Trans>Password settings content goes here.</Trans>
                      </p>
                    </TabsContent>
                  </Tabs>
                </CardContent>
              </Card>
            </div>
          </div>
        </div>
      </Layout>
    );
  } catch (error) {
    console.error('Error in Theme:', error);
    return (
      <div className='p-6'>
        <h1>Theme Demo</h1>
        <p>
          Error loading theme demo:{' '}
          {error instanceof Error ? error.message : 'Unknown error'}
        </p>
      </div>
    );
  }
}
