import { commands } from '@/bindings';

export async function openAppPermissionsReview(appId: string) {
  return await commands.appsStartSystemApp({
    kind: 'appUpdate',
    mode: 'reviewPermissions',
    appId,
  });
}
