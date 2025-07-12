import { loadCatalog } from '@/i18n';
import { createContext, ReactNode, useContext } from 'react';

const SUPPORTED_LANGUAGES = ['en-US', 'de-DE', 'zh-CN', 'es-MX'] as const;
export type SupportedLanguage = (typeof SUPPORTED_LANGUAGES)[number];
export interface LanguageContextType {
  locale: SupportedLanguage;
  changeLanguage: (newLocale: SupportedLanguage) => Promise<void>;
}

export const LanguageContext = createContext<LanguageContextType>({
  locale: 'en-US',
  changeLanguage: async () => {
    // Intentionally left blank
  },
});

export const getBrowserLanguage = (): SupportedLanguage => {
  const browserLang = navigator.language;
  return SUPPORTED_LANGUAGES.includes(browserLang as SupportedLanguage)
    ? (browserLang as SupportedLanguage)
    : 'en-US';
};

export function LanguageProvider({
  children,
  locale,
  setLocale,
}: {
  children: ReactNode;
  locale: SupportedLanguage;
  setLocale: (locale: SupportedLanguage) => void;
}) {
  const changeLanguage = async (newLocale: SupportedLanguage) => {
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
