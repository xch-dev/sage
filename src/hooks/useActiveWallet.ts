import { useEffect, useState } from 'react';
import { commands, KeyInfo } from '../bindings';
import { useErrors } from './useErrors';
import { useNavigate } from 'react-router-dom';

export function useActiveWallet() {
    const [wallet, setWallet] = useState<KeyInfo | null>(null);
    const [loading, setLoading] = useState(true);
    const { addError } = useErrors();
    const navigate = useNavigate();

    useEffect(() => {
        commands
            .getKey({})
            .then((data) => {
                setWallet(data.key);
                setLoading(false);
            })
            .catch((error) => {
                addError(error);
                setLoading(false);
            });
    }, [addError]);

    const redirectToLogin = () => navigate('/');

    return { wallet, loading, redirectToLogin };
} 