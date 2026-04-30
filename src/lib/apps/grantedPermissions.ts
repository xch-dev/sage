import type {
  SageGrantedPermissionsInput,
  SageGrantedPermissionsView,
} from '@/bindings';

export function grantedViewToInput(
  view: SageGrantedPermissionsView,
): SageGrantedPermissionsInput {
  return {
    capabilities: view.capabilities,
    network: {
      whitelist: view.network.whitelist
    },
  };
}

export function grantedInputToView(
  input: SageGrantedPermissionsInput,
): SageGrantedPermissionsView {
  return {
    capabilities: input.capabilities,
    network: {
      whitelist: input.network.whitelist,
    },
  };
}

export function emptyGrantedPermissionsInput(): SageGrantedPermissionsInput {
  return {
    capabilities: [],
    network: {
      whitelist: []
    },
  };
}

export function emptyGrantedPermissionsView(): SageGrantedPermissionsView {
  return {
    capabilities: [],
    network: {
      whitelist: [],
    },
  };
}
