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
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip';

interface NavProps {
  isCollapsed?: boolean;
}

export function Nav({ isCollapsed }: NavProps) {
  return (
    <nav className='grid gap-1 font-medium'>
      <NavLink
        url={'/wallet'}
        isCollapsed={isCollapsed}
        tooltip={<Trans>Wallet</Trans>}
      >
        <WalletIcon className='h-4 w-4' />
        {!isCollapsed && <Trans>Wallet</Trans>}
      </NavLink>
      <NavLink
        url={'/nfts'}
        isCollapsed={isCollapsed}
        tooltip={<Trans>NFTs</Trans>}
      >
        <Images className='h-4 w-4' />
        {!isCollapsed && <Trans>NFTs</Trans>}
      </NavLink>
      <NavLink
        url={'/dids'}
        isCollapsed={isCollapsed}
        tooltip={<Trans>Profiles</Trans>}
      >
        <SquareUserRound className='h-4 w-4' />
        {!isCollapsed && <Trans>Profiles</Trans>}
      </NavLink>
      <NavLink
        url={'/offers'}
        isCollapsed={isCollapsed}
        tooltip={<Trans>Offers</Trans>}
      >
        <RouteIcon className='h-4 w-4' />
        {!isCollapsed && <Trans>Offers</Trans>}
      </NavLink>
      <NavLink
        url={'/wallet/addresses'}
        isCollapsed={isCollapsed}
        tooltip={<Trans>Addresses</Trans>}
      >
        <BookUser className='h-4 w-4' />
        {!isCollapsed && <Trans>Addresses</Trans>}
      </NavLink>
      <NavLink
        url={'/transactions'}
        isCollapsed={isCollapsed}
        tooltip={<Trans>Transactions</Trans>}
      >
        <ArrowLeftRight className='h-4 w-4' />
        {!isCollapsed && <Trans>Transactions</Trans>}
      </NavLink>
    </nav>
  );
}

interface NavLinkProps extends PropsWithChildren {
  url: string;
  isCollapsed?: boolean;
  tooltip: React.ReactNode;
}

function NavLink({ url, children, isCollapsed, tooltip }: NavLinkProps) {
  const link = (
    <Link
      to={url}
      className={`flex items-center gap-3 rounded-lg px-3 py-1.5 text-muted-foreground transition-all hover:text-primary text-lg md:text-base ${
        isCollapsed ? 'justify-center' : ''
      }`}
    >
      {children}
    </Link>
  );

  if (isCollapsed) {
    return (
      <Tooltip>
        <TooltipTrigger asChild>{link}</TooltipTrigger>
        <TooltipContent side='right'>{tooltip}</TooltipContent>
      </Tooltip>
    );
  }

  return link;
}
