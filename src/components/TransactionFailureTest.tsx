import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { events } from '../bindings';

export function TransactionFailureTest() {
  const triggerTestEvent = () => {
    // Manually emit a test transaction failure event
    events.syncEvent.emit({
      type: 'transaction_failed',
      error: 'Test error message',
      transaction_id:
        '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
    });
  };

  return (
    <Card className='w-full max-w-md'>
      <CardHeader>
        <CardTitle>Transaction Failure Test</CardTitle>
      </CardHeader>
      <CardContent>
        <Button onClick={triggerTestEvent} variant='destructive'>
          Trigger Test Transaction Failure
        </Button>
        <p className='text-sm text-muted-foreground mt-2'>
          This will trigger a test transaction failure event to verify the toast
          notification and console logging works.
        </p>
      </CardContent>
    </Card>
  );
}
