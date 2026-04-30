import type {
  SageAppView,
  SystemSageAppView,
  UserSageAppView,
} from '@/bindings';
import {
  closeAppRuntime,
  ensureInlineRuntime,
  markRuntimeVisible,
} from '@/lib/apps/runtimeRegistry';

type AppLike = SageAppView | UserSageAppView | SystemSageAppView;

export async function restartAppRuntime(
  app: AppLike,
  options?: { visible?: boolean },
) {
  const visible = options?.visible ?? false;

  await closeAppRuntime(app.common.identity.id, { timeoutMs: 8000 });
  const runtime = await ensureInlineRuntime(app);
  await markRuntimeVisible(app.common.identity.id, visible);
  return runtime;
}
