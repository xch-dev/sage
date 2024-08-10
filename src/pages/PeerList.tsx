import { Delete, Star } from '@mui/icons-material';
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
} from '@mui/material';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { peerList } from '../commands';
import ListContainer from '../components/ListContainer';
import NavBar from '../components/NavBar';
import { PeerInfo } from '../models';

export default function NetworkList() {
  const navigate = useNavigate();

  const [peers, setPeers] = useState<PeerInfo[] | null>(null);

  useEffect(() => {
    peerList().then(setPeers);
  }, []);

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
        {peers !== null && (
          <List sx={{ width: '100%' }} component={Paper} disablePadding>
            {peers.map((peer, i) => (
              <PeerItem key={i} peer={peer} anyTrusted={anyTrusted} />
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

function PeerItem(props: { peer: PeerInfo; anyTrusted: boolean }) {
  return (
    <ListItem
      disablePadding
      secondaryAction={
        <Box display='flex' gap={1.5}>
          <IconButton edge='end'>
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
          secondary={props.peer.port}
          inset={!props.peer.trusted && props.anyTrusted}
        />
      </ListItemButton>
    </ListItem>
  );
}
