import { CatRecord, commands, events } from '@/bindings';
import Container from '@/components/Container';
import Header from '@/components/Header';
import Layout from '@/components/Layout';
import { DropdownSelector } from '@/components/selectors/DropdownSelector';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { useErrors } from '@/hooks/useErrors';
import { dbg, fromMojos, toMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { ArrowUpDown } from 'lucide-react';
import { useCallback, useEffect, useMemo, useState } from 'react';

interface Token {
  id: string;
  name: string;
  code: string;
  denom: number;
  icon: string;
}

interface Quote {
  combination_fee: number;
  suggested_tx_fee: number;
  from: string;
  to: string;
  from_amount: string;
  to_amount: string;
}

export default function Swap() {
  const { addError } = useErrors();

  const walletState = useWalletState();

  const [walletCats, setWalletCats] = useState<CatRecord[]>([]);
  const [tokens, setTokens] = useState<Token[]>([]);
  const [from, setFrom] = useState<Token | null>(null);
  const [fromAmount, setFromAmount] = useState('');
  const [to, setTo] = useState<Token | null>(null);
  const [toAmount, setToAmount] = useState('');
  const [quote, setQuote] = useState<Quote | null>(null);

  useEffect(() => {
    fetch('https://api.dexie.space/v1/swap/tokens')
      .then((res) => res.json())
      .then((data: { tokens: Token[] }) =>
        setTokens([
          {
            id: 'xch',
            name: 'Chia',
            code: walletState.sync.unit.ticker,
            denom: 12,
            icon: 'https://icons.dexie.space/xch.webp',
          },
          ...data.tokens.map((token) => ({
            ...token,
            denom: Math.log10(token.denom),
          })),
        ]),
      );
  }, [walletState.sync.unit.ticker]);

  const updateWalletCats = useCallback(() => {
    commands
      .getCats({})
      .then((data) => setWalletCats(data.cats))
      .catch(addError);
  }, [addError]);

  useEffect(() => {
    updateWalletCats();

    const unlisten = events.syncEvent.listen((event) => {
      if (event.payload.type === 'cat_info') {
        updateWalletCats();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [updateWalletCats]);

  const getQuote = async (
    amount: string,
    decimals: number,
    quote: 'from' | 'to',
  ): Promise<Quote | null> => {
    if (!from || !to || !amount.trim()) return null;

    const response = await fetch(
      dbg(
        `https://api.dexie.space/v1/swap/quote?from=${encodeURIComponent(from.code)}&to=${encodeURIComponent(to.code)}&${quote === 'from' ? 'to_amount' : 'from_amount'}=${toMojos(amount, decimals)}`,
      ),
    );

    const data = await response.json();

    return data.quote as Quote;
  };

  const updateAmount = async (amount: string, quote: 'from' | 'to') => {
    if (!from || !to || !amount.trim()) {
      setFromAmount('');
      setToAmount('');
      setQuote(null);
      return;
    }

    if (quote === 'from') setToAmount(amount);
    else setFromAmount(amount);

    const data = await getQuote(
      amount,
      quote === 'from' ? to.denom : from.denom,
      quote,
    );
    setQuote(data);

    console.log(data, to);

    if (data) {
      if (quote === 'from')
        setFromAmount(fromMojos(data.from_amount, from.denom).toString());
      else setToAmount(fromMojos(data.to_amount, to.denom).toString());
    }
  };

  const spendableCats: Set<string> = useMemo(() => {
    const spendable = new Set<string>();

    walletCats.forEach((cat) => {
      if (fromMojos(cat.balance, 3).isGreaterThan(0)) {
        spendable.add(cat.asset_id);
      }
    });

    if (fromMojos(walletState.sync.balance, 12).isGreaterThan(0)) {
      spendable.add('xch');
    }

    return spendable;
  }, [walletCats, walletState.sync.balance]);

  const flippable = !to || spendableCats.has(to.id);

  return (
    <Layout>
      <Header title={t`Swap`} />

      <Container className='flex flex-col gap-4 max-w-screen-sm'>
        <Card className='p-4 rounded-lg shadow-sm'>
          <SwapTokenSelector
            tokens={tokens.filter(
              (item) => item.id !== to?.id && spendableCats.has(item.id),
            )}
            token={from}
            setToken={setFrom}
            amount={fromAmount}
            setAmount={(amount) => updateAmount(amount, 'to')}
            label={t`You pay`}
          />

          <div className='flex items-center justify-center my-2'>
            <Button variant='ghost' size='icon' disabled={!flippable}>
              <ArrowUpDown
                className='w-6 h-6 cursor-pointer'
                onClick={() => {
                  setFrom(to);
                  setTo(from);
                  setFromAmount(toAmount);
                  setToAmount(fromAmount);
                }}
              />
            </Button>
          </div>

          <SwapTokenSelector
            tokens={tokens.filter((item) => item.id !== from?.id)}
            token={to}
            setToken={setTo}
            amount={toAmount}
            setAmount={(amount) => updateAmount(amount, 'from')}
            label={t`You receive`}
          />

          <div className='flex justify-center mt-4'>
            <Button className='w-full sm:w-1/2'>Swap</Button>
          </div>
        </Card>
      </Container>
    </Layout>
  );
}

interface SwapTokenSelectorProps {
  tokens: Token[];
  token: Token | null;
  setToken: (token: Token | null) => void;
  amount: string;
  setAmount: (amount: string) => void;
  label: string;
}

function SwapTokenSelector({
  tokens,
  token,
  setToken,
  amount,
  setAmount,
  label,
}: SwapTokenSelectorProps) {
  const [search, setSearch] = useState('');

  return (
    <div>
      <div className='flex items-center gap-2'>
        <div className='basis-1/2'>
          <DropdownSelector
            loadedItems={tokens.filter((token) => {
              if (!search) return true;
              return (
                token.name.toLowerCase().includes(search.toLowerCase()) ||
                token.code.toLowerCase().includes(search.toLowerCase()) ||
                token.id.toLowerCase() === search.toLowerCase()
              );
            })}
            page={1}
            onSelect={setToken}
            renderItem={(token) => (
              <div className='flex items-center gap-2'>
                <img
                  src={token.icon}
                  alt={token.code}
                  className='w-5 h-5 flex-shrink-0 rounded-full'
                />
                <span className='font-medium'>{token.code}</span>
                <span className='text-xs text-muted-foreground ml-auto truncate'>
                  {token.name}
                </span>
              </div>
            )}
            manualInput={
              <Input
                placeholder={t`Search by name or asset id`}
                value={search}
                onChange={(e) => {
                  const value = e.target.value;
                  setSearch(value);
                }}
              />
            }
          >
            <div className='pl-1 text-base'>
              {token ? (
                <div className='flex items-center gap-2'>
                  <img
                    src={token.icon}
                    alt={token.code}
                    className='w-6 h-6 flex-shrink-0 rounded-full'
                  />
                  <span>{token.code}</span>
                </div>
              ) : (
                <div className='flex items-center gap-2'>
                  <span className='text-muted-foreground'>{t`Select a token`}</span>
                </div>
              )}
            </div>
          </DropdownSelector>
        </div>

        <div className='relative basis-1/2'>
          <Input
            placeholder='0.0'
            value={amount}
            onChange={(e) => {
              const value = e.target.value;
              setAmount(value);
            }}
            className='h-12 text-base'
          />
          <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
            <span className='text-gray-500 text-sm' id='price-currency'>
              {label}
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}
