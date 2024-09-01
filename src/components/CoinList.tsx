import { DataGrid, GridRowSelectionModel } from '@mui/x-data-grid';
import BigNumber from 'bignumber.js';
import { useMemo } from 'react';
import { P2CoinData } from '../models';

export interface CoinListProps {
  coins: P2CoinData[];
  selectedCoins: GridRowSelectionModel;
  setSelectedCoins: React.Dispatch<React.SetStateAction<GridRowSelectionModel>>;
}

export default function CoinList(props: CoinListProps) {
  const rows = useMemo(() => {
    return props.coins.map((coin) => ({
      id: coin.coin_id,
      amount: coin.amount,
      confirmed: coin.created_height,
      spent: coin.spent_height,
      updated:
        coin.created_height && coin.spent_height
          ? Math.max(coin.created_height, coin.spent_height)
          : (coin.created_height ?? coin.spent_height),
    }));
  }, [props.coins]);

  return (
    <DataGrid
      sx={{ mt: 1, height: '100%' }}
      columns={[
        {
          field: 'id',
          headerName: 'Coin',
          width: 150,
          description: 'The coin id on the Chia blockchain',
        },
        {
          field: 'amount',
          headerName: 'Amount',
          width: 150,
          valueGetter: (_value, row) => new BigNumber(row.amount),
          description: 'The amount of XCH in this coin',
        },
        {
          field: 'confirmed',
          headerName: 'Confirmed',
          width: 150,
          description: 'The block height at which the coin was created',
        },
        {
          field: 'spent',
          headerName: 'Spent',
          width: 150,
          description: 'The block height at which the coin was spent',
        },
        {
          field: 'updated',
          headerName: 'Updated',
          width: 150,
          description: 'The last block height that this coin was updated',
        },
      ]}
      rows={rows}
      rowHeight={52}
      checkboxSelection
      onRowSelectionModelChange={(rows) => {
        props.setSelectedCoins(rows);
      }}
      rowSelectionModel={props.selectedCoins}
      initialState={{
        sorting: {
          sortModel: [{ field: 'updated', sort: 'desc' }],
        },
        pagination: {
          paginationModel: {
            pageSize: 10,
          },
        },
        filter: {
          filterModel: {
            items: [
              {
                id: 1,
                field: 'spent',
                operator: 'isEmpty',
              },
            ],
          },
        },
        columns: {
          columnVisibilityModel: {
            updated: false,
          },
        },
      }}
      pageSizeOptions={[10, 25, 50, 100]}
    />
  );
}
