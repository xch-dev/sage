import { commands } from '@/bindings';

export async function openAppUpdateReview(appId: string) {
  return await commands.appsStartSystemApp({
    kind: 'appUpdate',
    mode: 'reviewUpdate',
    appId,
  });
}

export async function openAppPermissionsReview(appId: string) {
  return await commands.appsStartSystemApp({
    kind: 'appUpdate',
    mode: 'reviewPermissions',
    appId,
  });
}
