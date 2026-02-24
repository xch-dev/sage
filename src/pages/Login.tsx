import SafeAreaView from '@/components/SafeAreaView';
import { WalletCard } from '@/components/WalletCard';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Skeleton } from '@/components/ui/skeleton';
import { useErrors } from '@/hooks/useErrors';
import {
  closestCenter,
  DndContext,
  DragEndEvent,
  DragOverlay,
  DragStartEvent,
  MouseSensor,
  TouchSensor,
  UniqueIdentifier,
  useSensor,
  useSensors,
} from '@dnd-kit/core';
import {
  arrayMove,
  rectSortingStrategy,
  SortableContext,
} from '@dnd-kit/sortable';
import { Trans } from '@lingui/react/macro';
import { platform } from '@tauri-apps/plugin-os';
import {
  ClockPlusIcon,
  CogIcon,
  EyeIcon,
  UserRoundKeyIcon,
  UserRoundPlusIcon,
  VaultIcon,
} from 'lucide-react';
import type { MouseEvent, TouchEvent } from 'react';
import { ForwardedRef, forwardRef, useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { commands, KeyInfo } from '../bindings';
import Container from '../components/Container';

const isMobile = platform() === 'ios' || platform() === 'android';

export default function Login() {
  const navigate = useNavigate();
  const { addError } = useErrors();
  const [keys, setKeys] = useState<KeyInfo[] | null>(null);

  useEffect(() => {
    commands
      .getKeys({})
      .then((data) => setKeys(data.keys))
      .catch(addError);
  }, [addError]);

  useEffect(() => {
    commands
      .getKey({})
      .then((data) => {
        if (data.key !== null) {
          navigate('/wallet');
        }
      })
      .catch((error) => {
        addError(error);
      });
  }, [navigate, addError]);

  const [activeId, setActiveId] = useState<UniqueIdentifier | null>(null);

  const mouseSensor = useSensor(MouseSensor, {
    activationConstraint: {
      distance: 10,
    },
  });
  const touchSensor = useSensor(TouchSensor, {
    activationConstraint: {
      delay: 250,
      tolerance: 5,
    },
  });
  const sensors = useSensors(mouseSensor, touchSensor);

  function handleDragStart(event: DragStartEvent) {
    const { active } = event;

    setActiveId(active.id);
  }

  function handleDragEnd(event: DragEndEvent) {
    const { active, over } = event;

    setActiveId(null);

    if (!keys || !over || active.id === over.id) return;

    const oldIndex = keys.findIndex((key) => key.fingerprint === active.id);
    const newIndex = keys.findIndex((key) => key.fingerprint === over.id);

    if (oldIndex === newIndex || oldIndex === -1 || newIndex === -1) return;

    setKeys(arrayMove(keys, oldIndex, newIndex));

    commands.moveKey(active.id as number, newIndex).catch(addError);
  }

  const activeKey = keys?.find((key) => key.fingerprint === activeId);

  return (
    <SafeAreaView>
      <div
        className={`flex-1 space-y-4 px-4 overflow-y-scroll ${
          isMobile ? '' : 'py-2 pb-4'
        }`}
      >
        <div className='flex items-center justify-between space-y-2'>
          {(keys?.length ?? 0) > 0 && (
            <>
              <h2 className='text-3xl font-bold tracking-tight'>
                <Trans>Wallets</Trans>
              </h2>
              <div className='flex items-center space-x-2'>
                <Button
                  variant='ghost'
                  size='icon'
                  onClick={() => navigate('/settings')}
                >
                  <CogIcon className='h-5 w-5' aria-hidden='true' />
                </Button>

                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button>
                      <Trans>Add Wallet</Trans>
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent className='w-40' align='start'>
                    <DropdownMenuGroup>
                      <DropdownMenuLabel>
                        <Trans>Standard</Trans>
                      </DropdownMenuLabel>

                      <DropdownMenuItem onClick={() => navigate('/create')}>
                        <UserRoundPlusIcon className='h-4 w-4 mr-2' />
                        <Trans>Create</Trans>
                      </DropdownMenuItem>

                      <DropdownMenuItem onClick={() => navigate('/import')}>
                        <UserRoundKeyIcon className='h-4 w-4 mr-2' />
                        <Trans>Import</Trans>
                      </DropdownMenuItem>
                    </DropdownMenuGroup>

                    <DropdownMenuSeparator />

                    <DropdownMenuGroup>
                      <DropdownMenuLabel>
                        <Trans>Vault</Trans>
                      </DropdownMenuLabel>

                      <DropdownMenuItem onClick={() => navigate('/create')}>
                        <VaultIcon className='h-4 w-4 mr-2' />
                        <Trans>Mint</Trans>
                      </DropdownMenuItem>

                      <DropdownMenuItem onClick={() => navigate('/import')}>
                        <ClockPlusIcon className='h-4 w-4 mr-2' />
                        <Trans>Recover</Trans>
                      </DropdownMenuItem>
                    </DropdownMenuGroup>

                    <DropdownMenuSeparator />

                    <DropdownMenuGroup>
                      <DropdownMenuLabel>
                        <Trans>Other</Trans>
                      </DropdownMenuLabel>

                      <DropdownMenuItem onClick={() => navigate('/create')}>
                        <EyeIcon className='h-4 w-4 mr-2' />
                        <Trans>Watch Address</Trans>
                      </DropdownMenuItem>
                    </DropdownMenuGroup>
                  </DropdownMenuContent>
                </DropdownMenu>
              </div>
            </>
          )}
        </div>
        {keys !== null ? (
          keys.length ? (
            <DndContext
              sensors={sensors}
              collisionDetection={closestCenter}
              onDragStart={handleDragStart}
              onDragEnd={handleDragEnd}
            >
              <SortableContext
                items={keys.map((key) => key.fingerprint)}
                strategy={rectSortingStrategy}
              >
                <div className='grid sm:grid-cols-2 md:grid-cols-3 gap-3'>
                  {keys.map((key) => (
                    <WalletCard
                      draggable
                      key={key.fingerprint}
                      info={key}
                      keys={keys}
                      setKeys={setKeys}
                    />
                  ))}
                </div>
              </SortableContext>
              <DragOverlay>
                {activeId && activeKey && (
                  <WalletCard info={activeKey} keys={keys} setKeys={setKeys} />
                )}
              </DragOverlay>
            </DndContext>
          ) : (
            <Welcome />
          )
        ) : (
          <SkeletonWalletList />
        )}
      </div>
    </SafeAreaView>
  );
}

export const Item = forwardRef(
  (
    { id, ...props }: { id: UniqueIdentifier },
    ref: ForwardedRef<HTMLDivElement>,
  ) => {
    return (
      <div {...props} ref={ref}>
        {id}
      </div>
    );
  },
);

Item.displayName = 'Item';

function SkeletonWalletList() {
  return (
    <div className='grid sm:grid-cols-2 md:grid-cols-3 gap-3 m-4'>
      {Array.from({ length: 3 }).map((_, i) => (
        // eslint-disable-next-line react/no-array-index-key
        <div key={i} className='w-full'>
          <Skeleton className='h-[100px] w-full' />
        </div>
      ))}
    </div>
  );
}

function Welcome() {
  const navigate = useNavigate();

  return (
    <Container>
      <div className='text-center text-6xl'>
        Sage <Trans>Wallet</Trans>
      </div>

      <div className='text-center mt-4'>
        <Trans>
          There aren&apos;t any wallets to log into yet. To get started, create
          a new wallet or import an existing one.
        </Trans>
      </div>

      <div className='flex justify-center gap-4 mt-6'>
        <Button variant='outline' onClick={() => navigate('/import')}>
          <Trans>Import Wallet</Trans>
        </Button>
        <Button onClick={() => navigate('/create')}>
          <Trans>Create Wallet</Trans>
        </Button>
      </div>
    </Container>
  );
}

const customHandleEvent = (element: HTMLElement | null) => {
  let cur = element;

  while (cur) {
    if (cur.dataset.noDnd) {
      return false;
    }
    cur = cur.parentElement;
  }

  return true;
};

MouseSensor.activators = [
  {
    eventName: 'onMouseDown',
    handler: ({ nativeEvent: event }: MouseEvent) =>
      customHandleEvent(event.target as HTMLElement),
  },
];

TouchSensor.activators = [
  {
    eventName: 'onTouchStart',
    handler: ({ nativeEvent: event }: TouchEvent) =>
      customHandleEvent(event.target as HTMLElement),
  },
];
