import Container from '@/components/Container';
import Header from '@/components/Header';
import { DidDisplay } from '@/components/DidDisplay';
import { useParams } from 'react-router-dom';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';

export default function Profile() {
  const { launcher_id: launcherId } = useParams();

  if (!launcherId) {
    return (
      <>
        <Header title={t`Invalid DID`} />
        <Container>
          <div className='text-center text-gray-500 dark:text-gray-400'>
            <Trans>Invalid DID ID</Trans>
          </div>
        </Container>
      </>
    );
  }

  return (
    <>
      <Header title={t`DID Profile`} />
      <Container>
        <div className='mx-auto sm:w-full md:w-[50%] max-w-[600px]'>
          <DidDisplay
            launcherId={launcherId}
            title={t`DID Profile`}
            showExternalLinks={true}
          />
        </div>
      </Container>
    </>
  );
}
