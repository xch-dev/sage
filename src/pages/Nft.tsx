import { Box, Typography, useTheme } from '@mui/material';
import Grid2 from '@mui/material/Unstable_Grid2/Grid2';
import { useEffect, useMemo, useState } from 'react';
import { useParams } from 'react-router-dom';
import { commands, events, NftRecord } from '../bindings';
import ListContainer from '../components/ListContainer';

export default function Nft() {
  const theme = useTheme();

  const { launcher_id: launcherId } = useParams();

  const [nft, setNft] = useState<NftRecord | null>(null);

  const updateNft = () => {
    commands.getNft(launcherId!).then((res) => {
      if (res.status === 'ok') {
        setNft(res.data);
      }
    });
  };

  useEffect(() => {
    updateNft();

    const unlisten = events.syncEvent.listen((event) => {
      if (event.payload.type === 'nft_update') {
        updateNft();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  });

  const metadata = useMemo(() => {
    if (!nft || !nft.metadata) return {};
    try {
      return JSON.parse(nft.metadata) ?? {};
    } catch {
      return {};
    }
  }, [nft]);

  return (
    <>
      <ListContainer>
        <Typography variant='h4' textAlign='center' mt={4}>
          {metadata.name ?? 'Unknown NFT'}
        </Typography>

        <Typography
          variant='subtitle1'
          textAlign='center'
          mt={1}
          sx={{ wordBreak: 'break-all' }}
        >
          {nft?.launcher_id}
        </Typography>

        <Grid2 container mt={2}>
          <Grid2 xs={12} md={6}>
            <Box sx={{ p: 1.5, width: '100%', mx: 'auto' }}>
              <img
                src={`data:${nft?.data_mime_type};base64,${nft?.data}`}
                style={{
                  width: '100%',
                  aspectRatio: 1,
                  borderRadius: theme.shape.borderRadius,
                }}
              />

              {metadata.description && (
                <>
                  <Typography variant='h6' mt={2}>
                    Description
                  </Typography>
                  <Typography variant='body1'>
                    {metadata.description}
                  </Typography>
                </>
              )}
            </Box>
          </Grid2>
          <Grid2 xs={12} md={6}>
            <Box sx={{ p: 1.5 }}>
              <Typography variant='h6' sx={{ m: 0 }}>
                Owner DID
              </Typography>
              <Typography variant='body1' sx={{ wordBreak: 'break-all' }}>
                {nft?.owner_did}
              </Typography>

              <Typography variant='h6' mt={2}>
                Address
              </Typography>
              <Typography variant='body1' sx={{ wordBreak: 'break-all' }}>
                {nft?.address}
              </Typography>

              <Typography variant='h6' mt={2}>
                Coin Id
              </Typography>
              <Typography variant='body1' sx={{ wordBreak: 'break-all' }}>
                {nft?.coin_id}
              </Typography>

              <Typography variant='h6' mt={2}>
                Royalties ({nft?.royalty_percent}%)
              </Typography>
              <Typography variant='body1' sx={{ wordBreak: 'break-all' }}>
                {nft?.royalty_address}
              </Typography>
            </Box>
          </Grid2>
        </Grid2>
      </ListContainer>
    </>
  );
}
