import { useLocalStorage } from 'usehooks-ts';

export default function DarkToggle() {
    const [dark, setDark] = useLocalStorage('dark', false);

    const toggle = () => {
        const html = document.querySelector('html')!;

        if (dark) html.classList.remove('dark');
        else html.classList.add('dark');

        setDark(!dark);
    };

    return <button onClick={toggle}>{dark ? 'SUN' : 'MOON'}</button>;
}
