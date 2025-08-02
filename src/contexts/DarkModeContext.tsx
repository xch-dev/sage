import { createContext } from 'react';

export interface DarkModeContext {
  toggle: () => void;
  dark: boolean;
  setDark: (dark: boolean) => void;
}

export const DarkModeContext = createContext<DarkModeContext>({
  toggle: () => {
    // Intentionally left blank
  },
  dark: false,
  setDark: () => {
    // Intentionally left blank
  },
});
