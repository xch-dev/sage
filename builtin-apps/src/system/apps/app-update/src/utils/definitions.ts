import type {
  SageAppCapabilityDefinitionView,
  UserBridgeCapability,
} from 'sage-system-app-sdk';

export function definitionMap(definitions: SageAppCapabilityDefinitionView[]) {
  return new Map(definitions.map((d) => [d.key as UserBridgeCapability, d]));
}

export function isUserGrantable(
  definitions: Map<UserBridgeCapability, SageAppCapabilityDefinitionView>,
  capability: UserBridgeCapability,
) {
  return definitions.get(capability)?.flags.userGrantable === true;
}
