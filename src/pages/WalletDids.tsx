import Container from '@/components/Container';
import Header from '@/components/Header';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { EyeOff, MoreVerticalIcon, PenIcon, UserRoundPlus } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { commands, DidRecord } from '../bindings';

export function WalletDids() {
  const navigate = useNavigate();

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
        {dids.length === 0 && (
          <Alert className='mb-4'>
            <UserRoundPlus className='h-4 w-4' />
            <AlertTitle>Create a profile?</AlertTitle>
            <AlertDescription>
              You do not currently have any DID profiles. Would you like to
              create one?
            </AlertDescription>
          </Alert>
        )}
        <Button onClick={() => navigate('create-profile')} className='mb-4'>
          Create Profile
        </Button>
        <div className='grid gap-4 md:grid-cols-2 md:gap-4 lg:grid-cols-3 xl:grid-cols-4'>
          {dids.map((did) => {
            return (
              <Card
                key={did.launcher_id}
                className='hover:-translate-y-0.5 duration-100 hover:shadow-md'
              >
                <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2 space-x-2'>
                  <CardTitle className='text-md font-medium truncate'>
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
        </div>
      </Container>
    </>
  );
}
