import type {
  SageAppCapabilityDefinitionView,
  SageAppCompatibility,
  SageAppPackageManifestPreview,
  SageAppUrlPreview,
} from 'sage-system-app-sdk';

export type InstallSource =
  | {
      kind: 'zip';
      zipPath: string;
      preview: SageAppPackageManifestPreview;
      compatibility: SageAppCompatibility;
    }
  | {
      kind: 'url';
      appUrl: string;
      preview: SageAppUrlPreview;
      compatibility: SageAppCompatibility;
    };

export type LoadState =
  | { kind: 'loading' }
  | { kind: 'selecting'; definitions: SageAppCapabilityDefinitionView[] }
  | {
      kind: 'review';
      definitions: SageAppCapabilityDefinitionView[];
      source: InstallSource;
    }
  | { kind: 'error'; error: string };
