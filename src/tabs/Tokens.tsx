import { Edit, MoreVert, Refresh } from '@mui/icons-material';
import {
  Avatar,
  Box,
  IconButton,
  ListItem,
  ListItemAvatar,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Menu,
  MenuItem,
  Paper,
  Typography,
  useMediaQuery,
  useTheme,
} from '@mui/material';
import { useEffect, useState } from 'react';
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
        .map((cat, i) => (
          <Token key={i} cat={cat} />
        ))}
    </Box>
  );
}

interface TokenProps {
  cat: CatRecord;
}

function Token(props: TokenProps) {
  const theme = useTheme();
  const md = useMediaQuery(theme.breakpoints.up('md'));
  const sm = useMediaQuery(theme.breakpoints.up('sm'));

  const chars = sm ? 16 : 8;

  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);
  const open = Boolean(anchorEl);

  const handleClick = (event: React.MouseEvent<HTMLButtonElement>) => {
    setAnchorEl(event.currentTarget);
  };

  const handleClose = () => {
    setAnchorEl(null);
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
                horizontal: 'left',
              }}
              transformOrigin={{
                vertical: 'top',
                horizontal: 'right',
              }}
              anchorEl={anchorEl}
              open={open}
              onClose={handleClose}
            >
              <MenuItem>
                <ListItemIcon>
                  <Edit fontSize='small' />
                </ListItemIcon>
                <ListItemText>Edit</ListItemText>
              </MenuItem>

              <MenuItem>
                <ListItemIcon>
                  <Refresh fontSize='small' />
                </ListItemIcon>
                <ListItemText>Redownload</ListItemText>
              </MenuItem>
            </Menu>
          </Box>
        }
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
    </Paper>
  );
}
