import { Box, Button, Paper, Typography, useTheme } from '@mui/material';
import Grid2 from '@mui/material/Unstable_Grid2/Grid2';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { commands, NftRecord } from '../bindings';

export function WalletNfts() {
  const [page, setPage] = useState(0);
  const [totalPages, setTotalPages] = useState(1);
  const [nfts, setNfts] = useState<NftRecord[]>([]);
  const [loading, setLoading] = useState(false);

  const updateNfts = async (page: number) => {
    return await commands
      .getNfts({ offset: page * 60, limit: 60 })
      .then((result) => {
        if (result.status === 'ok') {
          setNfts(result.data.items);
          setTotalPages(Math.max(1, Math.ceil(result.data.total / 60)));
        } else {
          throw new Error('Failed to get NFTs');
        }
      });
  };

  const nextPage = () => {
    if (loading) return;
    setLoading(true);
    updateNfts(page + 1)
      .then(() => setPage(page + 1))
      .finally(() => {
        setLoading(false);
      });
  };

  const previousPage = () => {
    if (loading) return;
    setLoading(true);
    updateNfts(page - 1)
      .then(() => setPage(page - 1))
      .finally(() => {
        setLoading(false);
      });
  };

  useEffect(() => {
    updateNfts(0);
  }, []);

  useEffect(() => {
    const interval = setInterval(() => {
      updateNfts(page);
    }, 5000);

    return () => {
      clearInterval(interval);
    };
  }, [page]);

  return (
    <>
      <Box display='flex' justifyContent='center' alignItems='center' gap={2}>
        <Button
          variant='outlined'
          onClick={() => previousPage()}
          disabled={page === 0}
        >
          Previous
        </Button>
        <Typography variant='body1'>
          Page {page + 1} of {totalPages}
        </Typography>
        <Button
          variant='outlined'
          onClick={() => nextPage()}
          disabled={page >= totalPages - 1}
        >
          Next
        </Button>
      </Box>
      <Grid2 mt={3} container spacing={2}>
        {nfts.map((nft, i) => (
          <Nft nft={nft} key={i} />
        ))}
      </Grid2>
    </>
  );
}

function Nft({ nft }: { nft: NftRecord }) {
  const navigate = useNavigate();
  const theme = useTheme();

  let json: any = {};

  if (nft.metadata) {
    try {
      json = JSON.parse(nft.metadata);
    } catch (error) {
      console.error(error);
    }
  }

  return (
    <Grid2 xs={6} sm={4} md={4}>
      <Box position='relative' width='100%' height='100%'>
        <Button
          sx={{ padding: 0, width: '100%', height: '100%' }}
          onClick={() => navigate(`/wallet/nft/${nft.launcher_id}`)}
        >
          <img
            src={`data:${nft.data_mime_type};base64,${nft.data}`}
            style={{
              width: '100%',
              height: '100%',
              borderRadius: theme.shape.borderRadius,
            }}
          />

          <Paper
            sx={{
              position: 'absolute',
              bottom: 0,
              width: '100%',
              height: '40px',
              p: 1,
              borderTopLeftRadius: '0px',
              borderTopRightRadius: '0px',
              borderBottomLeftRadius: theme.shape.borderRadius,
              borderBottomRightRadius: theme.shape.borderRadius,
              textAlign: 'center',
              textTransform: 'none',
              overflow: 'none',
            }}
          >
            <Typography variant='body1' width='100%' height='100%'>
              {json.name ?? 'Unknown NFT'}
            </Typography>
          </Paper>
        </Button>
      </Box>
    </Grid2>
  );
}
