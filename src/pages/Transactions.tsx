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
import { useErrors } from '@/hooks/useErrors';
import { toDecimal } from '@/lib/utils';
import { useWalletState } from '@/state';
import { ReloadIcon } from '@radix-ui/react-icons';
import { FastForward, Info, MoreVerticalIcon } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { commands, PendingTransactionRecord } from '../bindings';

export function Transactions() {
  const walletState = useWalletState();

  const { addError } = useErrors();

  const [transactions, setTransactions] = useState<PendingTransactionRecord[]>(
    [],
  );

  const updateTransactions = useCallback(async () => {
    commands
      .getPendingTransactions({})
      .then((data) => setTransactions(data.transactions))
      .catch(addError);
  }, [addError]);

  useEffect(() => {
    updateTransactions();

    const interval = setInterval(() => {
      updateTransactions();
    }, 5000);

    return () => {
      clearInterval(interval);
    };
  }, [updateTransactions]);

  return (
    <>
      <Header title='Transactions'>
        <ReceiveAddress />
      </Header>
      <Container>
        <Alert className='mb-4'>
          <Info className='h-4 w-4' />
          <AlertTitle>Note</AlertTitle>
          <AlertDescription>
            This only shows transactions initiated by this app that are
            currently pending in the mempool.
          </AlertDescription>
        </Alert>
        {transactions.map((transaction, i) => {
          return (
            <Card key={i}>
              <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2'>
                <CardTitle className='text-2xl font-medium'>
                  Transaction {transaction.transaction_id.slice(0, 16)}
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
                        <ReloadIcon className='mr-2 h-4 w-4' />
                        <span>Resubmit</span>
                      </DropdownMenuItem>
                      <DropdownMenuItem
                        className='cursor-pointer'
                        onClick={(e) => {
                          e.stopPropagation();
                        }}
                      >
                        <FastForward className='mr-2 h-4 w-4' />
                        <span>Increase Fee</span>
                      </DropdownMenuItem>
                    </DropdownMenuGroup>
                  </DropdownMenuContent>
                </DropdownMenu>
              </CardHeader>
              <CardContent>
                <div className='text-sm truncate'>
                  With a fee of{' '}
                  {toDecimal(transaction.fee, walletState.sync.unit.decimals)}{' '}
                  {walletState.sync.unit.ticker}
                </div>
              </CardContent>
            </Card>
          );
        })}
      </Container>
    </>
  );
}
