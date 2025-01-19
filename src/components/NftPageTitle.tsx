import { t } from '@lingui/core/macro';
import { NftCollectionRecord, DidRecord } from '../bindings';
import { NftGroupMode } from '@/hooks/useNftParams';
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip';

interface NftPageTitleProps {
  collectionId?: string | null;
  collection?: NftCollectionRecord | null;
  ownerDid?: string | null;
  owner?: DidRecord | null;
  minterDid?: string | null;
  group?: NftGroupMode;
}

function getGroupTitle(params: NftPageTitleProps) {
  const { collectionId, collection, ownerDid, owner, minterDid, group } =
    params;

  if (collectionId) {
    if (collection?.name === 'Uncategorized') return t`No Collection`;
    return collection?.name ?? t`No Collection`;
  }
  if (ownerDid) {
    return owner?.name ?? t`Untitled Profile`;
  }
  if (minterDid) {
    if (minterDid === 'No did') return t`Unknown Minter`;
    return minterDid;
  }

  switch (group) {
    case NftGroupMode.Collection:
      return t`Collections`;
    case NftGroupMode.OwnerDid:
      return t`Owner Profiles`;
    case NftGroupMode.MinterDid:
      return t`Minters`;
    default:
      return t`NFTs`;
  }
}

export function NftPageTitle(props: NftPageTitleProps) {
  const title = getGroupTitle(props);

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <div className='truncate max-w-[300px]'>{title}</div>
      </TooltipTrigger>
      <TooltipContent>{title}</TooltipContent>
    </Tooltip>
  );
}
