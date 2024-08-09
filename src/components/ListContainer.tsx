import { Box } from '@mui/material';
import { PropsWithChildren } from 'react';

export default function ListContainer(props: PropsWithChildren<object>) {
  return (
    <Box
      sx={{
        mx: { xs: 2, sm: 'auto' },
        mt: { xs: 2, sm: 4 },
        width: { sm: '540px', md: '700px' },
      }}
    >
      {props.children}
    </Box>
  );
}
