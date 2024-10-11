import Container from '@/components/Container';
import Header from '@/components/Header';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { EyeOff, MoreVerticalIcon, PenIcon } from 'lucide-react';
import { useEffect, useState } from 'react';
import { commands, DidRecord } from '../bindings';

export function WalletDids() {
  const [dids, setDids] = useState<DidRecord[]>([]);

  const updateDids = async () => {
    return await commands.getDids().then((result) => {
      if (result.status === 'ok') {
        setDids(result.data);
      } else {
        throw new Error('Failed to get DIDs');
      }
    });
  };

  useEffect(() => {
    updateDids();

    const interval = setInterval(() => {
      updateDids();
    }, 5000);

    return () => {
      clearInterval(interval);
    };
  }, []);

  return (
    <>
      <Header title='Profiles'>
        <ReceiveAddress />
      </Header>
      <Container>
        {dids.map((did, i) => {
          return (
            <Card
              key={i}
              className='hover:-translate-y-0.5 duration-100 hover:shadow-md'
            >
              <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2'>
                <CardTitle className='text-xl font-medium'>
                  Untitled Profile
                </CardTitle>
                <DropdownMenu>
                  <DropdownMenuTrigger asChild className='-mr-2.5'>
                    <Button variant='ghost' size='icon'>
                      <MoreVerticalIcon className='h-5 w-5' />
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align='end'>
                    <DropdownMenuGroup>
                      <DropdownMenuItem
                        className='cursor-pointer'
                        onClick={(e) => {
                          e.stopPropagation();
                        }}
                      >
                        <PenIcon className='mr-2 h-4 w-4' />
                        <span>Rename</span>
                      </DropdownMenuItem>
                      <DropdownMenuItem
                        className='cursor-pointer text-red-600 focus:text-red-500'
                        onClick={(e) => {
                          e.stopPropagation();
                        }}
                      >
                        <EyeOff className='mr-2 h-4 w-4' />
                        <span>Hide</span>
                      </DropdownMenuItem>
                    </DropdownMenuGroup>
                  </DropdownMenuContent>
                </DropdownMenu>
              </CardHeader>
              <CardContent>
                <div className='text-sm font-medium truncate'>
                  {did.encoded_id}
                </div>
              </CardContent>
            </Card>
          );
        })}
      </Container>
    </>
  );
}
