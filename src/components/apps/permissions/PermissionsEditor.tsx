import { useEffect, useState } from 'react';
import { PermissionsEditor as SharedPermissionsEditor } from '@sage-app/ui';
import type {
  SageAppCapabilityDefinitionView,
  SageGrantedPermissionsInput,
  SageGrantedPermissionsView,
  SystemSageAppView,
  UserSageAppView,
} from '@/bindings';
import { commands } from '@/bindings';

interface Props {
  app: UserSageAppView | SystemSageAppView;
  grantedPermissions: SageGrantedPermissionsView;
  onGrantedPermissionsChange?: (next: SageGrantedPermissionsInput) => void;
  editable?: boolean;
}

export function PermissionsEditor(props: Props) {
  const [capabilityDefinitions, setCapabilityDefinitions] = useState<
    SageAppCapabilityDefinitionView[]
  >([]);

  useEffect(() => {
    let cancelled = false;

    void commands
      .getUserCapabilityDefinitions()
      .then((definitions) => {
        if (!cancelled) {
          setCapabilityDefinitions(definitions);
        }
      })
      .catch((err) => {
        console.error('Failed to load capability definitions:', err);
      });

    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <SharedPermissionsEditor
      {...props}
      capabilityDefinitions={capabilityDefinitions}
    />
  );
}
