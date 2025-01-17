import { PropsWithChildren } from 'react';

export default function Container(
  props: PropsWithChildren<{ className?: string }>,
) {
  return (
    <main
      className={`flex-1 overflow-y-auto p-4 md:p-6 md:pt-4 pb-4 ${props.className ?? ''}`}
    >
      {props.children}
    </main>
  );
}
