import { Button } from '@mui/material';
import NavBar from '../components/NavBar';

export default function Login() {
  return (
    <>
      <NavBar />
      <Button variant="contained">Hello</Button>
      <div className="grid grid-cols-3 gap-3 p-3">
        <WalletItem />
        <WalletItem />
        <WalletItem />
      </div>
    </>
  );
}

function WalletItem() {
  return (
    <div
      className="
            cursor-pointer p-3 rounded-md border-solid border
            drop-shadow-md
            "
    >
      <h2 className="text-xl">Wallet</h2>
      <p className="text-md">529734</p>
    </div>
  );
}
