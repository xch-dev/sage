import type {
  SageAppCapabilityDefinitionView,
  SageAppPackageManifest,
  SageAppUrlPreview,
} from '@sage-system-app/sdk';

export type InstallSource =
  | { kind: 'zip'; zipPath: string; manifest: SageAppPackageManifest }
  | { kind: 'url'; appUrl: string; preview: SageAppUrlPreview };

export type LoadState =
  | { kind: 'loading' }
  | { kind: 'selecting'; definitions: SageAppCapabilityDefinitionView[] }
  | {
      kind: 'review';
      definitions: SageAppCapabilityDefinitionView[];
      source: InstallSource;
    }
  | { kind: 'error'; error: string };
