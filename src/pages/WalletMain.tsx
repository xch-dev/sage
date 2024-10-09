import { GridRowSelectionModel } from '@mui/x-data-grid';
import BigNumber from 'bignumber.js';
import { useEffect, useRef, useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { CatRecord, commands, events } from '../bindings';
import { useWalletState } from '../state';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import Header from '@/components/Header';
import Container from '@/components/Container';

export function MainWallet() {
  const navigate = useNavigate();
  const walletState = useWalletState();

  const [cats, setCats] = useState<CatRecord[]>([]);

  const updateCats = () => {
    commands.getCats().then(async (result) => {
      if (result.status === 'ok') {
        setCats(result.data);
      }
    });
  };

  useEffect(() => {
    updateCats();

    const unlisten = events.syncEvent.listen((event) => {
      if (event.payload.type === 'cat_update') {
        updateCats();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, []);

  const [selectedCoins, setSelectedCoins] = useState<GridRowSelectionModel>([]);
  const [isCombineOpen, setCombineOpen] = useState(false);
  const [isSplitOpen, setSplitOpen] = useState(false);
  const [combineFee, setCombineFee] = useState('');
  const [splitOutputCount, setSplitOutputCount] = useState('');
  const [splitFee, setSplitFee] = useState('');

  const splitFeeRef = useRef<HTMLInputElement>(null);

  const combineFeeNum = BigNumber(combineFee);
  const splitOutputCountNum = BigNumber(splitOutputCount);
  const splitFeeNum = BigNumber(splitFee);

  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);
  const open = Boolean(anchorEl);

  const handleClick = (event: React.MouseEvent<HTMLButtonElement>) => {
    setAnchorEl(event.currentTarget);
  };

  const handleClose = () => {
    setAnchorEl(null);
  };

  const canCombine =
    !combineFeeNum.isNaN() &&
    BigNumber(walletState.sync.balance).gte(combineFeeNum);

  const combine = () => {
    if (!canCombine) return;

    commands.combine(selectedCoins as string[], combineFee).then((result) => {
      setCombineOpen(false);

      if (result.status === 'ok') {
        setSelectedCoins([]);
      }
    });
  };

  const outputCountValid =
    !splitOutputCountNum.isNaN() &&
    splitOutputCountNum.isLessThanOrEqualTo(4294967295);
  const splitFeeValid =
    !splitFeeNum.isNaN() &&
    BigNumber(walletState.sync.balance).gte(splitFeeNum);
  const canSplit = outputCountValid && splitFeeValid;

  const split = () => {
    if (!canSplit) return;

    commands
      .split(
        selectedCoins as string[],
        splitOutputCountNum.toNumber(),
        splitFee,
      )
      .then((result) => {
        setSplitOpen(false);

        if (result.status === 'ok') {
          setSelectedCoins([]);
        }
      });
  };

  return (
    <>
      <Header title='Wallet' />
      <Container>
        <div className='grid gap-4 md:grid-cols-2 md:gap-8 lg:grid-cols-3 xl:grid-cols-4'>
          <Link to={`/wallet/token/xch`}>
            <Card className='hover:-translate-y-0.5 duration-100 hover:shadow-md'>
              <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2'>
                <CardTitle className='text-sm font-medium'>Chia</CardTitle>

                <img
                  alt={`XCH logo`}
                  className='h-6 w-6'
                  src='https://icons.dexie.space/xch.webp'
                />
              </CardHeader>
              <CardContent>
                <div className='text-2xl font-medium'>
                  {walletState.sync.balance} {walletState.sync.unit.ticker}
                </div>
              </CardContent>
            </Card>
          </Link>
          {cats.map((cat) => (
            <Link to={`/wallet/token/${cat.asset_id}`}>
              <Card className='hover:-translate-y-0.5 duration-100 hover:shadow-md'>
                <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2 space-x-2'>
                  <CardTitle className='text-sm font-medium truncate'>
                    {cat.name}
                  </CardTitle>

                  {cat.icon_url && (
                    <img
                      alt={`${cat.asset_id} logo`}
                      className='h-6 w-6'
                      src={cat.icon_url}
                    />
                  )}
                </CardHeader>
                <CardContent>
                  <div className='text-2xl font-medium'>
                    {cat.balance} {cat.ticker}
                  </div>
                </CardContent>
              </Card>
            </Link>
          ))}
        </div>
      </Container>
    </>
  );
}
