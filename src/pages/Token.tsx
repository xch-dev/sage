import { Box, Button, Typography } from '@mui/material';
import { GridRowSelectionModel } from '@mui/x-data-grid';
import { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { CatRecord, CoinRecord, commands, events } from '../bindings';
import CoinList from '../components/CoinList';
import ListContainer from '../components/ListContainer';

export default function Token() {
  const navigate = useNavigate();
  const { asset_id: assetId } = useParams();

  const [cat, setCat] = useState<CatRecord | null>(null);
  const [coins, setCoins] = useState<CoinRecord[]>([]);
  const [selectedCoins, setSelectedCoins] = useState<GridRowSelectionModel>([]);

  const updateCoins = () => {
    commands.getCatCoins(assetId!).then((res) => {
      if (res.status === 'ok') {
        setCoins(res.data);
      }
    });
  };

  useEffect(() => {
    updateCoins();

    const unlisten = events.syncEvent.listen((event) => {
      if (
        event.payload.type === 'coin_update' ||
        event.payload.type === 'puzzle_update'
      ) {
        updateCoins();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, []);

  const updateCat = () => {
    commands.getCat(assetId!).then((res) => {
      if (res.status === 'ok') {
        setCat(res.data);
      }
    });
  };

  useEffect(() => {
    updateCat();

    const unlisten = events.syncEvent.listen((event) => {
      if (event.payload.type === 'cat_update') {
        updateCat();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, []);

  return (
    <>
      <ListContainer>
        <Typography variant='h5' fontSize={30} textAlign='center'>
          {cat?.balance ?? 'Loading'} {cat?.ticker ?? 'CAT'}
        </Typography>

        <Box display='flex' gap={2} mt={2}>
          <Button
            variant='outlined'
            size='large'
            sx={{ flexGrow: 1 }}
            onClick={() => navigate('/send-cat/' + assetId)}
          >
            Send
          </Button>
          <Button
            variant='outlined'
            size='large'
            sx={{ flexGrow: 1 }}
            onClick={() => navigate('/receive')}
          >
            Receive
          </Button>
        </Box>

        <Box height={350} position='relative' mt={2}>
          <CoinList
            coins={coins}
            selectedCoins={selectedCoins}
            setSelectedCoins={setSelectedCoins}
          />
        </Box>
      </ListContainer>
    </>
  );
}
