export function resolveBackgroundTintWithAlpha(alpha: number = 0.85): string {
  const tint = resolveBackgroundTint();

  return colorWithAlpha(tint, alpha);
}

function resolveBackgroundTint(): string {
  const root = getComputedStyle(document.documentElement);

  const candidates = [
    root.getPropertyValue('--background').trim(),
    root.getPropertyValue('--secondary').trim(),
    root.getPropertyValue('--muted').trim(),
    root.getPropertyValue('--card').trim(),
  ];

  return (
    candidates.find(
      (value) =>
        value.length > 0 &&
        value !== 'transparent' &&
        value !== 'rgba(0, 0, 0, 0)',
    ) ?? '#1d2530'
  );
}

function colorWithAlpha(color: string, alpha: number): string {
  if (color.startsWith('#')) {
    return `color-mix(in srgb, ${color} ${alpha * 100}%, transparent)`;
  }

  if (color.startsWith('rgb') || color.startsWith('hsl')) {
    return `color-mix(in srgb, ${color} ${alpha * 100}%, transparent)`;
  }

  // HSL channel format, e.g. "222 47% 11%"
  return `hsl(${color} / ${alpha})`;
}
