import { OptionRecord } from '@/bindings';

interface OptionGridViewProps {
  options: OptionRecord[];
  updateOptions: () => void;
  showHidden: boolean;
}

export function OptionGridView({ options, showHidden }: OptionGridViewProps) {
  const visibleOptions = showHidden
    ? options
    : options.filter((option) => option.visible);

  return (
    <div className='mt-4 grid gap-4 md:gap-4 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4'>
      {visibleOptions.map((option) => (
        <div key={option.launcher_id}>{option.name}</div>
      ))}
    </div>
  );
}
