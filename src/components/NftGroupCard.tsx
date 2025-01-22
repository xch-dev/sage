import { DidRecord, NftCollectionRecord } from '@/bindings';
import { NftGroupMode } from '@/hooks/useNftParams';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import {
  EyeIcon,
  EyeOff,
  Icon,
  icons,
  LibraryBig,
  MoreVerticalIcon,
  Paintbrush,
  UserIcon,
} from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { Button } from './ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from './ui/dropdown-menu';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from './ui/tooltip';

interface NftGroupCardProps {
  type: 'collection' | 'did';
  groupMode?: NftGroupMode;
  item: NftCollectionRecord | DidRecord;
  updateNfts: (page: number) => void;
  page: number;
  onToggleVisibility?: () => void;
  isLoading?: boolean;
  error?: Error;
}

export function NftGroupCard({
  type,
  groupMode,
  item,
  updateNfts,
  page,
  onToggleVisibility,
  isLoading,
  error,
}: NftGroupCardProps) {
  const navigate = useNavigate();
  const isCollection = type === 'collection';
  const allowToggleVisibility = false; //isCollection && onToggleVisibility; // not implemented yet
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
      <Trans>Unknown Minter</Trans>
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

  if (error) {
    return (
      <div
        className='border border-red-200 dark:border-red-800 rounded-lg p-4'
        role='alert'
      >
        <p className='text-red-600 dark:text-red-400'>{error.message}</p>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div
        className='border border-neutral-200 dark:border-neutral-800 rounded-lg p-4 animate-pulse'
        aria-label={isCollection ? t`Loading collection` : t`Loading profile`}
      >
        <div className='aspect-square bg-neutral-100 dark:bg-neutral-800 rounded-t-lg' />
        <div className='h-6 bg-neutral-100 dark:bg-neutral-800 rounded mt-4' />
      </div>
    );
  }

  return (
    <div
      onClick={() => {
        updateNfts(page);
        navigate(getLinkPath());
      }}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          updateNfts(page);
          navigate(getLinkPath());
        }
      }}
      role='button'
      tabIndex={0}
      className='cursor-pointer group border border-neutral-200 dark:border-neutral-800 rounded-lg focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary'
      aria-label={
        isCollection
          ? `Collection ${item.name || t`Unnamed`}`
          : groupMode === NftGroupMode.OwnerDid
            ? `Profile ${item.name || t`Untitled Profile`}`
            : `Minter ${item.name || t`Unknown Minter`}`
      }
      aria-current={
        window.location.pathname === getLinkPath() ? 'page' : undefined
      }
    >
      <div className='overflow-hidden rounded-t-lg relative'>
        {isCollection ? (
          <div
            className='bg-neutral-100 dark:bg-neutral-800 flex items-center justify-center aspect-square'
            aria-hidden='true'
          >
            {item?.icon ? (
              <img
                src={item.icon}
                alt={t`Icon for ${item.name}`}
                className='object-cover h-full w-full'
                aria-hidden='true'
              />
            ) : (
            <LibraryBig
              className='h-12 w-12 text-neutral-400 dark:text-neutral-600'
              aria-hidden='true'
            />
            )}

          </div>
        ) : (
          <div
            className='bg-neutral-100 dark:bg-neutral-800 flex items-center justify-center aspect-square'
            aria-hidden='true'
          >
            {groupMode === NftGroupMode.OwnerDid ? (
              <UserIcon
                className='h-12 w-12 text-neutral-400 dark:text-neutral-600'
                aria-hidden='true'
              />
            ) : (
              <Paintbrush
                className='h-12 w-12 text-neutral-400 dark:text-neutral-600'
                aria-hidden='true'
              />
            )}
          </div>
        )}
      </div>
      <div className='border-t border-neutral-200 bg-white text-neutral-950 shadow dark:border-neutral-800 dark:bg-neutral-900 dark:text-neutral-50 text-md flex items-center justify-between rounded-b-lg p-2 pl-3'>
        <span className='truncate'>
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <h3 className='font-medium leading-none truncate block'>
                  {item.name ?? getDefaultName()}
                </h3>
              </TooltipTrigger>
              <TooltipContent>
                <p>{item.name ?? getDefaultName()}</p>
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>

          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <p
                  className='text-xs text-muted-foreground truncate'
                  aria-label={`ID: ${getId()}`}
                >
                  {getId()}
                </p>
              </TooltipTrigger>
              <TooltipContent>
                <p>{getId()}</p>
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
        </span>

        {allowToggleVisibility && onToggleVisibility && (
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant='ghost'
                size='icon'
                onClick={(e) => e.stopPropagation()}
                aria-label={
                  item.visible ? t`Hide collection` : t`Show collection`
                }
                aria-expanded='false'
              >
                <MoreVerticalIcon className='h-5 w-5' aria-hidden='true' />
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
                    <EyeOff className='mr-2 h-4 w-4' aria-hidden='true' />
                  ) : (
                    <EyeIcon className='mr-2 h-4 w-4' aria-hidden='true' />
                  )}
                  <span>{item.visible ? t`Hide` : t`Show`}</span>
                </DropdownMenuItem>
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>
        )}
      </div>
    </div>
  );
}
