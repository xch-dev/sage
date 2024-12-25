import { loadCatalog } from '@/i18n';
import { createContext, useContext, useState, ReactNode } from 'react';

export interface LanguageContextType {
  locale: string;
  changeLanguage: (newLocale: string) => Promise<void>;
}

export const LanguageContext = createContext<LanguageContextType>({
  locale: '',
  changeLanguage: async () => {},
});

export function LanguageProvider({
  children,
  locale,
  setLocale,
}: {
  children: ReactNode;
  locale: string;
  setLocale: (locale: string) => void;
}) {
  const changeLanguage = async (newLocale: string) => {
    await loadCatalog(newLocale);
    setLocale(newLocale);
  };

  return (
    <LanguageContext.Provider value={{ locale, changeLanguage }}>
      {children}
    </LanguageContext.Provider>
  );
}

export function useLanguage() {
  return useContext(LanguageContext);
}
// import { loadCatalog } from '@/i18n';
// import { createContext, useContext, useState, ReactNode } from 'react';
// import { useLocalStorage } from 'usehooks-ts';

// interface LanguageContextType {
//   locale: string;
//   changeLanguage: (newLocale: string) => Promise<void>;
// }

// const LanguageContext = createContext<LanguageContextType | undefined>(
//   undefined,
// );

// export function LanguageProvider({ children }: { children: ReactNode }) {
//   const [locale, setLocale] = useLocalStorage('locale', 'en-US');

//   const changeLanguage = async (newLocale: string) => {
//     await loadCatalog(newLocale);
//     setLocale(newLocale);
//   };

//   return (
//     <LanguageContext.Provider value={{ locale, changeLanguage }}>
//       {children}
//     </LanguageContext.Provider>
//   );
// }

// export function useLanguage() {
//   const context = useContext(LanguageContext);
//   if (!context) {
//     throw new Error('useLanguage must be used within a LanguageProvider');
//   }
//   return context;
// }
