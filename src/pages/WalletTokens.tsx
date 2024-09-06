import { Edit, MoreVert, Refresh } from '@mui/icons-material';
import {
  Avatar,
  Box,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
  IconButton,
  ListItem,
  ListItemAvatar,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Menu,
  MenuItem,
  Paper,
  TextField,
  Typography,
  useMediaQuery,
  useTheme,
} from '@mui/material';
import { useEffect, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { CatRecord, commands, events } from '../bindings';

export function WalletTokens() {
  const [cats, setCats] = useState<CatRecord[]>([]);

  const updateCats = () => {
    commands.getCats().then((result) => {
      if (result.status === 'ok') {
        setCats(result.data);
      }
    });
  };

  useEffect(() => {
    updateCats();

    const unlisten = events.syncEvent.listen((event) => {
      if (event.payload.type === 'cat_update') {
        updateCats();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, []);

  return (
    <Box display='flex' flexDirection='column' gap={1.5}>
      {cats
        .sort((a, b) => {
          if (a.name && b.name) {
            return a.name.localeCompare(b.name);
          } else if (a.name) {
            return -1;
          } else {
            return 1;
          }
        })
        .map((cat) => (
          <Token key={cat.asset_id} cat={cat} updateCats={updateCats} />
        ))}
    </Box>
  );
}

interface TokenProps {
  cat: CatRecord;
  updateCats: () => void;
}

function Token(props: TokenProps) {
  const navigate = useNavigate();
  const theme = useTheme();
  const md = useMediaQuery(theme.breakpoints.up('md'));
  const sm = useMediaQuery(theme.breakpoints.up('sm'));

  const chars = sm ? 16 : 8;

  const [isEditOpen, setEditOpen] = useState(false);
  const [newName, setNewName] = useState('');
  const [newTicker, setNewTicker] = useState('');

  const tickerRef = useRef<HTMLInputElement>(null);

  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);
  const open = Boolean(anchorEl);

  const handleClick = (event: React.MouseEvent<HTMLButtonElement>) => {
    setAnchorEl(event.currentTarget);
    event.preventDefault();
    event.stopPropagation();
  };

  const handleClose = () => {
    setAnchorEl(null);
  };

  const redownload = () => {
    commands.removeCatInfo(props.cat.asset_id).then((res) => {
      if (res.status === 'ok') {
        props.updateCats();
      }
    });

    handleClose();
  };

  const edit = () => {
    if (!newName || !newTicker) return;

    props.cat.name = newName;
    props.cat.ticker = newTicker;

    commands.updateCatInfo(props.cat).then((res) => {
      if (res.status === 'ok') {
        props.updateCats();
      }
    });

    setEditOpen(false);
  };

  return (
    <Paper>
      <ListItem
        disablePadding
        secondaryAction={
          <Box display='flex'>
            <IconButton edge='end' onClick={handleClick}>
              <MoreVert />
            </IconButton>

            <Menu
              anchorOrigin={{
                vertical: 'center',
                horizontal: 'center',
              }}
              transformOrigin={{
                vertical: 'top',
                horizontal: 'right',
              }}
              anchorEl={anchorEl}
              open={open}
              onClose={handleClose}
            >
              <MenuItem
                onClick={() => {
                  setEditOpen(true);
                  handleClose();
                }}
              >
                <ListItemIcon>
                  <Edit fontSize='small' />
                </ListItemIcon>
                <ListItemText>Edit</ListItemText>
              </MenuItem>

              <MenuItem onClick={redownload}>
                <ListItemIcon>
                  <Refresh fontSize='small' />
                </ListItemIcon>
                <ListItemText>Redownload</ListItemText>
              </MenuItem>
            </Menu>
          </Box>
        }
        onClick={() => {
          if (open) return;
          navigate(`/wallet/token/${props.cat.asset_id}`);
        }}
      >
        <ListItemButton>
          <ListItemAvatar>
            {props.cat.icon_url && (
              <Avatar
                alt={props.cat.ticker ?? undefined}
                src={props.cat.icon_url}
              />
            )}
          </ListItemAvatar>
          <ListItemText
            primary={props.cat.name ?? 'Unknown CAT'}
            secondary={
              <Typography
                sx={{ display: 'inline' }}
                component='span'
                variant='body2'
                color='text.primary'
              >
                {md
                  ? props.cat.asset_id
                  : `${props.cat.asset_id.slice(0, chars)}..${props.cat.asset_id.slice(-chars)}`}
              </Typography>
            }
          />
        </ListItemButton>
      </ListItem>

      <Dialog
        open={isEditOpen}
        onClose={() => {
          setNewName('');
          setNewTicker('');
          setEditOpen(false);
        }}
      >
        <DialogTitle>Edit Token</DialogTitle>
        <DialogContent>
          <DialogContentText>
            Enter the new display details for this token.
          </DialogContentText>
          <TextField
            label='Name'
            variant='standard'
            margin='dense'
            required
            fullWidth
            autoFocus
            value={newName}
            onChange={(event) => setNewName(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === 'Enter') {
                event.preventDefault();
                tickerRef.current?.focus();
              }
            }}
          />

          <TextField
            inputRef={tickerRef}
            label='Ticker'
            variant='standard'
            margin='dense'
            required
            fullWidth
            value={newTicker}
            onChange={(event) => setNewTicker(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === 'Enter') {
                event.preventDefault();
                edit();
              }
            }}
          />
        </DialogContent>
        <DialogActions>
          <Button
            onClick={(event) => {
              event.preventDefault();
              setNewName('');
              setEditOpen(false);
            }}
          >
            Cancel
          </Button>
          <Button
            onClick={(event) => {
              event.preventDefault();
              edit();
            }}
            autoFocus
            disabled={!newName || !newTicker}
          >
            Submit
          </Button>
        </DialogActions>
      </Dialog>
    </Paper>
  );
}
