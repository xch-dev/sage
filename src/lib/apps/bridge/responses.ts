import type {
  SageBridgeSuccessResponse,
  SageBridgeVersion,
} from '@sage-app/sdk';

const BRIDGE_VERSION: SageBridgeVersion = 'v1';

export function success(
  id: string,
  result: unknown,
): SageBridgeSuccessResponse {
  return {
    bridgeVersion: BRIDGE_VERSION,
    id,
    ok: true,
    result,
  };
}
