import {
  ArrowLeftRight,
  BookUser,
  Images,
  RouteIcon,
  SquareUserRound,
  WalletIcon,
} from 'lucide-react';
import { PropsWithChildren } from 'react';
import { Link } from 'react-router-dom';
import { Trans } from '@lingui/react/macro';

export function Nav() {
  return (
    <nav className='grid gap-1 font-medium'>
      <NavLink url={'/wallet'}>
        <WalletIcon className='h-4 w-4' />
        <Trans>Wallet</Trans>
      </NavLink>
      <NavLink url={'/nfts'}>
        <Images className='h-4 w-4' />
        <Trans>NFTs</Trans>
      </NavLink>
      <NavLink url={'/dids'}>
        <SquareUserRound className='h-4 w-4' />
        <Trans>Profiles</Trans>
      </NavLink>
      <NavLink url={'/offers'}>
        <RouteIcon className='h-4 w-4' />
        <Trans>Offers</Trans>
      </NavLink>
      <NavLink url={'/wallet/addresses'}>
        <BookUser className='h-4 w-4' />
        <Trans>Addresses</Trans>
      </NavLink>
      <NavLink url={'/transactions'}>
        <ArrowLeftRight className='h-4 w-4' />
        <Trans>Transactions</Trans>
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
      className='flex items-center gap-3 rounded-lg px-3 py-1.5 text-muted-foreground transition-all hover:text-primary text-lg md:text-base'
    >
      {children}
    </Link>
  );
}
