import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { commands, Error } from '../bindings';
import Container from '../components/Container';
import ErrorDialog from '../components/ErrorDialog';
import Form, { FormValue } from '../components/Form';
import NavBar from '../components/NavBar';
import { useWalletState } from '../state';

export default function IssueCat() {
  const navigate = useNavigate();
  const walletState = useWalletState();

  const [values, setValues] = useState({
    name: '',
    amount: '',
    fee: '',
    multiIssuance: false,
  });
  const [error, setError] = useState<Error | null>(null);

  const issue = () => {
    commands.issueCat(values.name, values.amount, values.fee).then((result) => {
      if (result.status === 'error') {
        console.error(result.error);
        setError(result.error);
        return;
      }

      navigate('/wallet/tokens');
    });
  };

  return (
    <>
      <NavBar label='Issue CAT' back={() => navigate(-1)} />

      <Container>
        <Form
          submitName='Issue CAT'
          fields={[
            { id: 'name', type: 'text', label: 'Name' },
            {
              id: 'amount',
              type: 'amount',
              label: 'Amount',
              unit: { ticker: 'CAT', decimals: 3 },
            },
            {
              id: 'fee',
              type: 'amount',
              label: 'Fee',
              unit: walletState.sync.unit,
            },
            {
              id: 'multiIssuance',
              type: 'option',
              label: 'Allow more to be issued later',
            },
          ]}
          values={values}
          setValues={setValues as (values: Record<string, FormValue>) => void}
          onSubmit={issue}
        />
      </Container>

      <ErrorDialog error={error} setError={setError} />
    </>
  );
}
