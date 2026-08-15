import { useEffect, useMemo, useState } from 'react';
import type {
  AppUpdateReviewContext,
  SageAppCapabilityDefinitionView,
  SageGrantedPermissionsInput,
  UserSageAppView,
} from '@sage-system-app/sdk';
import { definitionMap } from '../utils/definitions';
import { nextPermissionsForUpdate } from '../utils/permissions';

export function useUpdatePermissions(args: {
  app: UserSageAppView;
  context: AppUpdateReviewContext | null;
  definitions: SageAppCapabilityDefinitionView[];
}) {
  const { app, context, definitions } = args;

  const [grantedPermissions, setGrantedPermissions] =
    useState<SageGrantedPermissionsInput>(app.common.grantedPermissions);

  const definitionsByKey = useMemo(
    () => definitionMap(definitions),
    [definitions],
  );

  useEffect(() => {
    if (!context) {
      setGrantedPermissions(app.common.grantedPermissions);
      return;
    }

    const next = nextPermissionsForUpdate({
      app,
      context,
      definitionsByKey,
    });

    if (next) {
      setGrantedPermissions(next);
    }
  }, [app, context, definitionsByKey]);

  return {
    grantedPermissions,
    setGrantedPermissions,
  };
}
