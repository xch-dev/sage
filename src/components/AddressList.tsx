import { DataGrid } from '@mui/x-data-grid';
import { useMemo } from 'react';

export interface AddressListProps {
  addresses: string[];
}

export default function AddressList(props: AddressListProps) {
  const rows = useMemo(() => {
    return props.addresses.map((address, i) => ({
      id: i,
      address,
    }));
  }, [props.addresses]);

  return (
    <DataGrid
      sx={{ height: '100%', mt: 4 }}
      columns={[
        {
          field: 'id',
          headerName: 'Index',
          width: 100,
          description: 'The derivation index of this address',
        },
        {
          field: 'address',
          headerName: 'Address',
          width: 400,
          description: 'The address on the Chia blockchain',
        },
      ]}
      rows={rows}
      rowHeight={52}
      rowSelection={false}
      initialState={{
        sorting: {
          sortModel: [{ field: 'index', sort: 'asc' }],
        },
        pagination: {
          paginationModel: {
            pageSize: 25,
          },
        },
      }}
      pageSizeOptions={[25, 50, 100]}
    />
  );
}
