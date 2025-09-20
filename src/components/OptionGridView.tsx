import { OptionRecord } from '@/bindings';
import { useOptionActions } from '@/hooks/useOptionActions';
import { OptionCard } from './OptionCard';

interface OptionGridViewProps {
  options: OptionRecord[];
  updateOptions: () => void;
  showHidden: boolean;
}

export function OptionGridView({
  options,
  updateOptions,
  showHidden,
}: OptionGridViewProps) {
  const { actionHandlers, dialogs } = useOptionActions(updateOptions);

  const visibleOptions = showHidden
    ? options
    : options.filter((option) => option.visible);

  return (
    <>
      <div className='mt-4 grid gap-4 md:gap-4 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4'>
        {visibleOptions.map((option) => (
          <OptionCard
            key={option.launcher_id}
            option={option}
            actionHandlers={actionHandlers}
          />
        ))}
      </div>
      {dialogs}
    </>
  );
}
