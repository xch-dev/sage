import {
  HandCoins,
  Images,
  LucideProps,
  SquareUserRound,
  WalletIcon,
} from 'lucide-react';
import { ForwardRefExoticComponent, RefAttributes } from 'react';

export type NavItem = NavLink | NavSeparator;

export interface NavLink {
  type: 'link';
  label: string;
  url: string;
  icon: ForwardRefExoticComponent<
    Omit<LucideProps, 'ref'> & RefAttributes<SVGSVGElement>
  >;
}

export interface NavSeparator {
  type: 'separator';
}

export const navItems: NavItem[] = [
  {
    type: 'link',
    label: 'Wallet',
    url: '/wallet',
    icon: WalletIcon,
  },
  {
    type: 'link',
    label: 'NFTs',
    url: '/nfts',
    icon: Images,
  },
  {
    type: 'link',
    label: 'Profiles',
    url: '/dids',
    icon: SquareUserRound,
  },
  {
    type: 'separator',
  },
  {
    type: 'link',
    label: 'Issue Token',
    url: '/wallet/issue-token',
    icon: HandCoins,
  },
];
