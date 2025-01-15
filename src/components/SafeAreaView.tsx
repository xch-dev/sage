import { useInsets } from '@/contexts/SafeAreaContext';
import { PropsWithChildren } from 'react';

export default function SafeAreaView(props: PropsWithChildren<object>) {
  const insets = useInsets();

  return (
    <div
      className='flex flex-col h-screen overflow-hidden bg-background '
      style={{
        paddingTop:
          insets.top !== 0 ? `${insets.top + 8}px` : 'env(safe-area-inset-top)',
        paddingBottom: insets.bottom
          ? `${insets.bottom}px`
          : 'env(safe-area-inset-bottom)',
      }}
    >
      {props.children}
    </div>
  );
}
