import {
  createContext,
  ReactNode,
  useCallback,
  useEffect,
  useState,
} from 'react';
import {
  checkSecureElementSupport,
  deleteKey,
  generateSecureKey,
  KeyInfo,
  listKeys,
  SecureElementBacking,
} from 'tauri-plugin-secure-element-api';

export type AuthenticationMode = 'none' | 'pinOrBiometric' | 'biometricOnly';

export interface SecureElementContextType {
  isSupported: boolean;
  canEnforceBiometricOnly: boolean;
  strongest: SecureElementBacking;
  isEmulated: boolean;
  keys: KeyInfo[];
  selectedKey: KeyInfo | null;
  isLoading: boolean;
  error: string | null;
  createKey: (name: string, authMode?: AuthenticationMode) => Promise<KeyInfo>;
  deleteKey: (keyName: string) => Promise<void>;
  refreshKeys: () => Promise<void>;
  selectKey: (keyName: string | null) => void;
}

export const SecureElementContext = createContext<
  SecureElementContextType | undefined
>(undefined);

export function SecureElementProvider({ children }: { children: ReactNode }) {
  const [isSupported, setIsSupported] = useState(false);
  const [canEnforceBiometricOnly, setCanEnforceBiometricOnly] = useState(false);
  const [strongest, setStrongest] = useState<SecureElementBacking>('none');
  const [isEmulated, setIsEmulated] = useState(false);
  const [keys, setKeys] = useState<KeyInfo[]>([]);
  const [selectedKey, setSelectedKey] = useState<KeyInfo | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Check for secure element support and load keys on mount
  useEffect(() => {
    const initialize = async () => {
      try {
        setIsLoading(true);
        setError(null);

        // Check support
        const support = await checkSecureElementSupport();
        const supported = support.strongest !== 'none';
        setIsSupported(supported);
        setCanEnforceBiometricOnly(support.canEnforceBiometricOnly);
        setStrongest(support.strongest);
        setIsEmulated(support.emulated);
        // Load keys if supported
        if (supported) {
          const keyList = await listKeys();
          setKeys(keyList);
        }
      } catch (err) {
        // If any error occurs, device does not support secure keys
        setIsSupported(false);
        setError(
          err instanceof Error ? err.message : 'Secure element not supported',
        );
      } finally {
        setIsLoading(false);
      }
    };

    initialize();
  }, []);

  const createKey = useCallback(
    async (
      name: string,
      authMode: AuthenticationMode = 'pinOrBiometric',
    ): Promise<KeyInfo> => {
      if (!isSupported) {
        throw new Error('Secure element is not supported on this device');
      }

      try {
        setError(null);

        // Check if key with this name already exists
        const existingKeys = await listKeys(name);
        const existingKey = existingKeys.find((key) => key.keyName === name);
        if (existingKey) {
          throw new Error(`Key with name "${name}" already exists`);
        }

        // Create the key
        const newKey = await generateSecureKey(name, authMode);

        // Refresh the keys list
        const keyList = await listKeys();
        setKeys(keyList);

        // Select the newly created key
        setSelectedKey(newKey);

        return newKey;
      } catch (err) {
        const errorMessage =
          err instanceof Error ? err.message : 'Failed to create secure key';
        setError(errorMessage);
        throw err;
      }
    },
    [isSupported],
  );

  const deleteKeyHandler = useCallback(
    async (keyName: string): Promise<void> => {
      if (!isSupported) {
        throw new Error('Secure element is not supported on this device');
      }

      try {
        setError(null);

        const success = await deleteKey(keyName);
        if (!success) {
          throw new Error('Failed to delete key');
        }

        // Refresh the keys list
        const keyList = await listKeys();
        setKeys(keyList);

        // Clear selection if the deleted key was selected
        if (selectedKey?.keyName === keyName) {
          setSelectedKey(null);
        }
      } catch (err) {
        const errorMessage =
          err instanceof Error ? err.message : 'Failed to delete secure key';
        setError(errorMessage);
        throw err;
      }
    },
    [isSupported, selectedKey],
  );

  const refreshKeys = useCallback(async (): Promise<void> => {
    if (!isSupported) {
      return;
    }

    try {
      setError(null);
      const keyList = await listKeys();
      setKeys(keyList);

      // Update selected key if it still exists
      if (selectedKey) {
        const updatedKey = keyList.find(
          (key) => key.keyName === selectedKey.keyName,
        );
        if (updatedKey) {
          setSelectedKey(updatedKey);
        } else {
          setSelectedKey(null);
        }
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to refresh keys');
    }
  }, [isSupported, selectedKey]);

  const selectKey = useCallback(
    (keyName: string | null): void => {
      if (keyName === null) {
        setSelectedKey(null);
        return;
      }

      const key = keys.find((k) => k.keyName === keyName);
      if (key) {
        setSelectedKey(key);
      } else {
        setError(`Key "${keyName}" not found`);
      }
    },
    [keys],
  );

  return (
    <SecureElementContext.Provider
      value={{
        isSupported,
        canEnforceBiometricOnly,
        strongest,
        isEmulated,
        keys,
        selectedKey,
        isLoading,
        error,
        createKey,
        deleteKey: deleteKeyHandler,
        refreshKeys,
        selectKey,
      }}
    >
      {children}
    </SecureElementContext.Provider>
  );
}
