import type {
  AppPermissionsReviewContext,
  AppUpdateReviewContext,
  SageAppCapabilityDefinitionView,
  SystemWalletView,
  UserSageAppView,
} from 'sage-system-app-sdk';

export type Mode = 'review-update' | 'review-permissions';

export type LoadState =
  | { kind: 'loading' }
  | { kind: 'error'; error: string }
  | {
      kind: 'ready';
      mode: 'review-permissions';
      app: UserSageAppView;
      updateContext: null;
      permissionsContext: AppPermissionsReviewContext;
      definitions: SageAppCapabilityDefinitionView[];
      wallets: SystemWalletView[];
    }
  | {
      kind: 'ready';
      mode: 'review-update';
      updateContext: AppUpdateReviewContext;
      permissionsContext: null;
      definitions: SageAppCapabilityDefinitionView[];
      wallets: SystemWalletView[];
    };
