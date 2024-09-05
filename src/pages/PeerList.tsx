import { Delete, Star } from '@mui/icons-material';
import {
  Box,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
  FormControlLabel,
  IconButton,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Paper,
  Switch,
  Tooltip,
  Typography,
} from '@mui/material';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { commands, PeerRecord } from '../bindings';
import ListContainer from '../components/ListContainer';
import NavBar from '../components/NavBar';

export default function NetworkList() {
  const navigate = useNavigate();

  const [peers, setPeers] = useState<PeerRecord[] | null>(null);

  const updatePeers = () => {
    commands.getPeers().then((res) => {
      if (res.status === 'ok') {
        setPeers(res.data);
      }
    });

    const interval = setInterval(() => {
      commands.getPeers().then((res) => {
        if (res.status === 'ok') {
          setPeers(res.data);
        }
      });
    }, 1000);

    return () => {
      clearInterval(interval);
    };
  };

  useEffect(updatePeers, []);

  const anyTrusted =
    peers === null ? false : peers.some((peer) => peer.trusted);

  return (
    <>
      <NavBar
        label='Peer List'
        back={() => {
          navigate(-1);
        }}
      />

      <ListContainer>
        <Typography variant='h5' textAlign='center'>
          Connected to {peers?.length ?? 0} peers
        </Typography>

        {peers !== null && (
          <List sx={{ mt: 2, width: '100%' }} component={Paper} disablePadding>
            {peers
              .sort((a, b) => a.ip_addr.localeCompare(b.ip_addr))
              .map((peer, i) => (
                <PeerItem
                  key={i}
                  peer={peer}
                  anyTrusted={anyTrusted}
                  updatePeers={updatePeers}
                />
              ))}
          </List>
        )}

        <Button variant='contained' fullWidth sx={{ mt: 2 }}>
          Add Peer
        </Button>
      </ListContainer>
    </>
  );
}

function PeerItem(props: {
  peer: PeerRecord;
  anyTrusted: boolean;
  updatePeers: () => void;
}) {
  const [removeOpen, setRemoveOpen] = useState(false);
  const [ban, setBan] = useState(false);

  return (
    <ListItem
      disablePadding
      secondaryAction={
        <Box display='flex' gap={1.5}>
          <IconButton edge='end' onClick={() => setRemoveOpen(true)}>
            <Delete />
          </IconButton>
        </Box>
      }
    >
      <ListItemButton>
        {props.peer.trusted && (
          <ListItemIcon>
            <Star />
          </ListItemIcon>
        )}
        <ListItemText
          primary={props.peer.ip_addr}
          secondary={`Port ${props.peer.port} | Peak height ${props.peer.peak_height}`}
          inset={!props.peer.trusted && props.anyTrusted}
        />
      </ListItemButton>

      <Dialog open={removeOpen} onClose={() => setRemoveOpen(false)}>
        <DialogTitle>Are you sure you want to remove the peer?</DialogTitle>
        <DialogContent>
          <DialogContentText>
            This will remove the peer from your connections. If you are
            currently syncing against this peer, a new one will be used to
            replace it.
          </DialogContentText>

          <FormControlLabel
            sx={{ mt: 2 }}
            control={
              <Switch
                checked={ban}
                onChange={(event) => setBan(event.target.checked)}
              />
            }
            label={
              <Tooltip
                title='Will temporarily prevent the peer from being connected to.'
                placement='bottom-start'
                enterDelay={750}
              >
                <span>Ban peer temporarily</span>
              </Tooltip>
            }
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setRemoveOpen(false)}>Cancel</Button>
          <Button
            onClick={() => {
              setRemoveOpen(false);
              commands.removePeer(props.peer.ip_addr, ban).then((res) => {
                if (res.status === 'ok') {
                  props.updatePeers();
                }
              });
            }}
            autoFocus
          >
            Confirm
          </Button>
        </DialogActions>
      </Dialog>
    </ListItem>
  );
}
