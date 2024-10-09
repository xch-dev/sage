import { PropsWithChildren } from 'react';

export default function Container(
  props: PropsWithChildren<{ className?: string }>,
) {
  return (
    <main className={'flex-1 overflow-y-auto p-4 lg:p-6 ' + props.className}>
      {props.children}
    </main>
  );
}
