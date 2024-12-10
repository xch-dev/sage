import { PropsWithChildren } from 'react';

export default function SafeAreaView(props: PropsWithChildren<object>) {
  return (
    <div className='flex flex-col h-screen overflow-hidden pt-[env(safe-area-inset-top)]'>
      {props.children}
    </div>
  );
}
