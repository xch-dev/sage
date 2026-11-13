import { useContext } from 'react';
import { NftPriceContext } from '../contexts/NftPriceContext';

export function useNftPrices() {
  const context = useContext(NftPriceContext);
  if (context === undefined) {
    throw new Error('useNftPrices must be used within an NftPriceProvider');
  }
  return context;
}
