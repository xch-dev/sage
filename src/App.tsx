import {
    Route,
    RouterProvider,
    createHashRouter,
    createRoutesFromElements,
} from 'react-router-dom';
import Login from './pages/Login';
import Wallet from './pages/Wallet';
import Welcome from './pages/Welcome';

const router = createHashRouter(
    createRoutesFromElements(
        <Route path="/" element={<Welcome />}>
            <Route path="wallet" element={<Wallet />} />
            <Route path="login" element={<Login />} />
        </Route>,
    ),
);

export default function App() {
    return <RouterProvider router={router} />;
}
