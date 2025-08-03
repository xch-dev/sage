import Container from '@/components/Container';
import Header from '@/components/Header';
import { Profile } from '@/components/Profile';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Switch } from '@/components/ui/switch';
import { useDids } from '@/hooks/useDids';
import { t } from '@lingui/core/macro';
import { Plural, Trans } from '@lingui/react/macro';
import { UserPlusIcon, UserRoundPlus } from 'lucide-react';
import { useState } from 'react';
import { useNavigate } from 'react-router-dom';

export function DidList() {
  const navigate = useNavigate();
  const { dids, updateDids } = useDids();
  const didsCount = dids.length;
  const [showHidden, setShowHidden] = useState(false);

  const visibleDids = showHidden ? dids : dids.filter((did) => did.visible);
  const hasHiddenDids = dids.findIndex((did) => !did.visible) > -1;

  return (
    <>
      <Header title={t`Profiles`}>
        <ReceiveAddress />
      </Header>
      <Container>
        <Button onClick={() => navigate('/dids/create')}>
          <UserPlusIcon className='h-4 w-4 mr-2' />
          <Trans>Create Profile</Trans>
        </Button>

        {hasHiddenDids && (
          <div className='flex items-center gap-2 my-4'>
            <label htmlFor='viewHidden'>
              <Trans>View hidden</Trans>
            </label>
            <Switch
              id='viewHidden'
              checked={showHidden}
              onCheckedChange={(value) => setShowHidden(value)}
            />
          </div>
        )}

        {didsCount === 0 && (
          <Alert className='mt-4'>
            <UserRoundPlus className='h-4 w-4' />
            <AlertTitle>
              <Trans>Create a profile?</Trans>
            </AlertTitle>
            <AlertDescription>
              <Plural
                value={didsCount}
                one='You do not currently have a DID profile. Would you like to create one?'
                other='You do not currently have any DID profiles. Would you like to create one?'
              />
            </AlertDescription>
          </Alert>
        )}

        <div className='mt-4 grid gap-4 md:gap-4 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4'>
          {visibleDids.map((did) => (
            <Profile
              key={did.launcher_id}
              did={did}
              variant='card'
              updateDids={updateDids}
              allowMintGardenProfile={true}
            />
          ))}
        </div>
      </Container>
    </>
  );
}
