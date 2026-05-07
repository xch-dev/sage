import type {
  AppPermissionsReviewContext,
  AppUpdateReviewContext,
  SageAppCapabilityDefinitionView,
  SystemWalletView,
  UserSageAppView,
} from '@sage-system-app/sdk';

export type Mode = 'review-update' | 'review-permissions';

export type LoadState =
  | { kind: 'loading' }
  | { kind: 'error'; error: string }
  | {
      kind: 'ready';
      mode: Mode;
      app: UserSageAppView;
      updateContext: AppUpdateReviewContext | null;
      permissionsContext: AppPermissionsReviewContext | null;
      definitions: SageAppCapabilityDefinitionView[];
      wallets: SystemWalletView[];
    };
