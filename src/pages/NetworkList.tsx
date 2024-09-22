import { Delete, Edit, Star } from '@mui/icons-material';
import {
  Box,
  Button,
  IconButton,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Paper,
  useMediaQuery,
  useTheme,
} from '@mui/material';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { commands, Network, NetworkConfig } from '../bindings';
import ListContainer from '../components/ListContainer';
import NavBar from '../components/NavBar';
import { fetchState } from '../state';

export default function NetworkList() {
  const navigate = useNavigate();

  const [networks, setNetworks] = useState<Record<string, Network> | null>(
    null,
  );
  const [networkConfig, setNetworkConfig] = useState<NetworkConfig | null>(
    null,
  );

  useEffect(() => {
    commands.networkList().then((res) => {
      if (res.status === 'ok') {
        setNetworks(res.data);
      }
    });
    commands.networkConfig().then((res) => {
      if (res.status === 'ok') {
        setNetworkConfig(res.data);
      }
    });
  }, []);

  return (
    <>
      <NavBar
        label='Network List'
        back={() => {
          navigate(-1);
        }}
      />

      <ListContainer>
        <List sx={{ width: '100%' }} component={Paper} disablePadding>
          {networks === null
            ? null
            : Object.entries(networks).map(([networkId, network], i) => (
                <NetworkItem
                  key={i}
                  selectedNetworkId={networkConfig?.network_id ?? null}
                  networkId={networkId}
                  network={network}
                  switchNetwork={() => {
                    if (
                      !networkConfig ||
                      networkConfig.network_id === networkId
                    ) {
                      return;
                    }
                    commands.setNetworkId(networkId).then((result) => {
                      if (result.status === 'error') return;

                      setNetworkConfig({
                        ...networkConfig,
                        network_id: networkId,
                      });

                      fetchState();
                    });
                  }}
                />
              ))}
        </List>

        <Button variant='contained' fullWidth sx={{ mt: 2 }}>
          Add Network
        </Button>
      </ListContainer>
    </>
  );
}

function NetworkItem(props: {
  selectedNetworkId: string | null;
  networkId: string;
  network: Network;
  switchNetwork: () => void;
}) {
  const theme = useTheme();
  const sm = useMediaQuery(theme.breakpoints.up('sm'));
  const md = useMediaQuery(theme.breakpoints.up('md'));

  const selected = props.selectedNetworkId === props.networkId;
  const genesisChallenge = md
    ? props.network.genesis_challenge
    : props.network.genesis_challenge.slice(0, sm ? 16 : 8) +
      '...' +
      props.network.genesis_challenge.slice(sm ? -16 : -8);

  return (
    <ListItem
      disablePadding
      secondaryAction={
        <Box display='flex' gap={1.5}>
          <IconButton edge='end'>
            <Edit />
          </IconButton>
          <IconButton edge='end'>
            <Delete />
          </IconButton>
        </Box>
      }
    >
      <ListItemButton onClick={props.switchNetwork}>
        {selected && (
          <ListItemIcon>
            <Star />
          </ListItemIcon>
        )}
        <ListItemText
          primary={props.networkId}
          secondary={genesisChallenge}
          inset={!selected}
        />
      </ListItemButton>
    </ListItem>
  );
}
