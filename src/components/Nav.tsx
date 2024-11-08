import {
  CoinsIcon,
  ImagePlus,
  Images,
  SquareUserRound,
  UserPlus,
  WalletIcon,
} from 'lucide-react';
import { PropsWithChildren } from 'react';
import { Link } from 'react-router-dom';
import { Separator } from './ui/separator';

export function Nav() {
  return (
    <nav className='grid gap-1 text-md font-medium'>
      <NavLink url={'/wallet'}>
        <WalletIcon className='h-4 w-4' />
        Wallet
      </NavLink>
      <NavLink url={'/nfts'}>
        <Images className='h-4 w-4' />
        NFTs
      </NavLink>
      <NavLink url={'/dids'}>
        <SquareUserRound className='h-4 w-4' />
        Profiles
      </NavLink>
      {/* <NavLink url={'/offers'}>
        <ArrowLeftRight className='h-4 w-4' />
        Offers
      </NavLink> */}
      <Separator className='my-1' />
      <NavLink url={'/wallet/issue-token'}>
        <CoinsIcon className='h-4 w-4' />
        Issue Token
      </NavLink>
      <NavLink url={'/dids/create-profile'}>
        <UserPlus className='h-4 w-4' />
        Create Profile
      </NavLink>
      <NavLink url={'/nfts/mint-nft'}>
        <ImagePlus className='h-4 w-4' />
        Mint NFT
      </NavLink>
    </nav>
  );
}

interface NavLinkProps {
  url: string;
}

function NavLink({ url, children }: PropsWithChildren<NavLinkProps>) {
  return (
    <Link
      to={url}
      className='mx-[-0.65rem] md:mx-0 flex items-center gap-3 rounded-lg px-3 py-2 text-muted-foreground transition-all hover:text-primary'
    >
      {children}
    </Link>
  );
}
