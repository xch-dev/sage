import { Box } from '@mui/material';
import { GridRowSelectionModel } from '@mui/x-data-grid';
import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import { CoinRecord, commands, events } from '../bindings';
import CoinList from '../components/CoinList';
import ListContainer from '../components/ListContainer';

export default function Token() {
  const { asset_id: assetId } = useParams();

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
  });

  return (
    <>
      <ListContainer>
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
