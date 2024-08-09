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
import { useNavigate } from 'react-router-dom';
import ListContainer from '../components/ListContainer';
import NavBar from '../components/NavBar';

export default function NetworkList() {
  const navigate = useNavigate();

  return (
    <>
      <NavBar
        label='Peer List'
        back={() => {
          navigate(-1);
        }}
      />

      <ListContainer>
        <List
          sx={{ width: '100%', bgcolor: 'background.paper' }}
          component={Paper}
          disablePadding
        >
          <PeerItem peer={{ ipAddr: '127.0.0.1', port: 8444 }} />
        </List>

        <Button variant='contained' fullWidth sx={{ mt: 2 }}>
          Add Peer
        </Button>
      </ListContainer>
    </>
  );
}

function PeerItem(props: { peer: { ipAddr: string; port: number } }) {
  const trusted = true;

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
        {trusted && (
          <ListItemIcon>
            <Star />
          </ListItemIcon>
        )}
        <ListItemText
          primary={props.peer.ipAddr}
          secondary={props.peer.port}
          inset={!trusted}
        />
      </ListItemButton>
    </ListItem>
  );
}
