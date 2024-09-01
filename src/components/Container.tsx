import { Card } from '@mui/material';
import Grid2 from '@mui/material/Unstable_Grid2/Grid2';
import { PropsWithChildren } from 'react';

export default function Container(props: PropsWithChildren<object>) {
  return (
    <Grid2
      container
      spacing={0}
      direction='column'
      alignItems='center'
      justifyContent='center'
      m={2}
      mb={4}
      sx={{ marginTop: { xs: 2, sm: 4 } }}
    >
      <Grid2 xs={12} sm={8} md={7} lg={6}>
        <Card sx={{ p: 3 }}>{props.children}</Card>
      </Grid2>
    </Grid2>
  );
}
