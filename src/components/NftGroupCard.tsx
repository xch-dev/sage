import { NftCollectionRecord, DidRecord } from '@/bindings';
import { NftGroupMode } from '@/hooks/useNftParams';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import {
  EyeIcon,
  EyeOff,
  MoreVerticalIcon,
  UserIcon,
  Paintbrush,
} from 'lucide-react';
import { Link } from 'react-router-dom';
import { Button } from './ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from './ui/dropdown-menu';
import collectionImage from '@/images/collection.png';

interface NftGroupCardProps {
  type: 'collection' | 'did';
  groupMode?: NftGroupMode;
  item: NftCollectionRecord | DidRecord;
  updateNfts: (page: number) => void;
  page: number;
  onToggleVisibility?: () => void;
}

export function NftGroupCard({
  type,
  groupMode,
  item,
  updateNfts,
  page,
  onToggleVisibility,
}: NftGroupCardProps) {
  const isCollection = type === 'collection';

  // Type guards to help TypeScript narrow the types
  const isDidRecord = (
    item: NftCollectionRecord | DidRecord,
  ): item is DidRecord => {
    return 'launcher_id' in item;
  };

  const isCollectionRecord = (
    item: NftCollectionRecord | DidRecord,
  ): item is NftCollectionRecord => {
    return 'collection_id' in item;
  };

  const getLinkPath = () => {
    if (isCollectionRecord(item)) {
      return `/nfts/collections/${item.collection_id}`;
    }
    if (isDidRecord(item)) {
      return groupMode === NftGroupMode.OwnerDid
        ? `/nfts/owners/${item.launcher_id}`
        : `/nfts/minters/${item.launcher_id}`;
    }
    return '';
  };

  const getDefaultName = () => {
    if (isCollection) {
      return t`Unnamed`;
    }
    return groupMode === NftGroupMode.OwnerDid ? (
      <Trans>Untitled Profile</Trans>
    ) : (
      <Trans>Unknown Creator</Trans>
    );
  };

  const getId = () => {
    if (isCollectionRecord(item)) {
      return item.collection_id;
    }
    if (isDidRecord(item)) {
      return item.launcher_id;
    }
    return '';
  };

  return (
    <div onClick={() => updateNfts(page)}>
      <Link
        to={getLinkPath()}
        className={`group${!item.visible ? ' opacity-50 grayscale' : ''} border border-neutral-200 rounded-lg dark:border-neutral-800`}
      >
        <div className='overflow-hidden rounded-t-lg relative'>
          {isCollection ? (
            <img
              alt={item.name ?? t`Unnamed`}
              loading='lazy'
              width='150'
              height='150'
              className='h-auto w-auto object-cover transition-all group-hover:scale-105 aspect-square color-[transparent]'
              src={collectionImage}
            />
          ) : (
            <div className='bg-neutral-100 dark:bg-neutral-800 flex items-center justify-center aspect-square'>
              {groupMode === NftGroupMode.OwnerDid ? (
                <UserIcon className='h-12 w-12 text-neutral-400 dark:text-neutral-600' />
              ) : (
                <Paintbrush className='h-12 w-12 text-neutral-400 dark:text-neutral-600' />
              )}
            </div>
          )}
        </div>
        <div className='border-t bg-white text-neutral-950 shadow dark:bg-neutral-900 dark:text-neutral-50 text-md flex items-center justify-between rounded-b-lg p-2 pl-3'>
          <span className='truncate'>
            <span className='font-medium leading-none truncate'>
              {item.name ?? getDefaultName()}
            </span>
            <p className='text-xs text-muted-foreground truncate'>{getId()}</p>
          </span>

          {isCollection && onToggleVisibility && (
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant='ghost' size='icon'>
                  <MoreVerticalIcon className='h-5 w-5' />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align='end'>
                <DropdownMenuGroup>
                  <DropdownMenuItem
                    className='cursor-pointer'
                    onClick={(e) => {
                      e.stopPropagation();
                      onToggleVisibility();
                    }}
                  >
                    {item.visible ? (
                      <EyeOff className='mr-2 h-4 w-4' />
                    ) : (
                      <EyeIcon className='mr-2 h-4 w-4' />
                    )}
                    <span>{item.visible ? t`Hide` : t`Show`}</span>
                  </DropdownMenuItem>
                </DropdownMenuGroup>
              </DropdownMenuContent>
            </DropdownMenu>
          )}
        </div>
      </Link>
    </div>
  );
}
