import { useInsets } from '@/contexts/SafeAreaContext';
import { PropsWithChildren } from 'react';

export default function SafeAreaView(props: PropsWithChildren<object>) {
  const insets = useInsets();
  console.log(insets.top);

  return (
    <div
      className='flex flex-col h-screen overflow-hidden'
      style={{
        paddingTop:
          insets.top !== 0 ? `${insets.top}px` : 'env(safe-area-inset-top)',
      }}
    >
      {props.children}
    </div>
  );
}
