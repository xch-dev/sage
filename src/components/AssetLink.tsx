import { AssetKind } from '@/bindings';
import { Link } from 'react-router-dom';

export interface AssetLinkProps {
  asset: {
    hash: string | null;
    name: string;
    kind: AssetKind;
  };
  className?: string;
}

export function AssetLink({ asset, className = '' }: AssetLinkProps) {
  if (asset.hash === null) {
    return null;
  }

  let link = null;
  switch (asset.kind) {
    case 'token':
      link = `/wallet/token/${asset.hash}`;
      break;
    case 'nft':
      link = `/nfts/${asset.hash}`;
      break;
    case 'option':
      link = `/options/${asset.hash}`;
      break;
    case 'did':
      // no dedicated profile page yet
      link = null;
      break;
    default:
      return null;
  }

  if (!link) {
    return <div className={`truncate ${className}`}>{asset.name}</div>;
  }

  return (
    <Link to={link} className={`hover:underline ${className}`}>
      <div className='truncate'>{asset.name}</div>
    </Link>
  );
}
