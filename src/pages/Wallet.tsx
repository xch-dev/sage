import Layout from '@/components/Layout';
import { Outlet } from 'react-router-dom';

export default function Wallet() {
  return (
    <Layout>
      <Outlet />
    </Layout>
  );
}
