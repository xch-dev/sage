import { Badge } from '@/components/ui/badge';

export function normalizeMnemonic(
  mnemonic: string,
  expectedWordCount: number,
): string | null {
  const words = mnemonic.trim().split(/\s+/).filter(Boolean);

  return words.length === expectedWordCount ? words.join(' ') : null;
}

export default function MnemonicDisplay({ mnemonic }: { mnemonic: string }) {
  const words = mnemonic ? mnemonic.split(' ') : [];

  return (
    <div className='flex flex-wrap' role='list'>
      {words.map((word, index) => (
        <Badge
          // Position is part of a mnemonic word's identity and permits repeats.
          // eslint-disable-next-line react/no-array-index-key
          key={`${word}-${index}`}
          variant='outline'
          className='py-1.5 px-2.5 m-0.5 rounded-lg font-medium'
          role='listitem'
        >
          {word}
        </Badge>
      ))}
    </div>
  );
}
