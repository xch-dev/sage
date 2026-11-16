import { sageLatestReleaseApiUrl, sageReleasePageUrl } from '@/lib/urls';

const REQUEST_TIMEOUT_MS = 15_000;

const VERSION_REGEX = /^v?(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?$/;

interface ParsedVersion {
  numbers: [number, number, number];
  prerelease: string | null;
}

export interface LatestRelease {
  /** The release's version, without the tag's leading `v`. */
  version: string;
  /** The release's page on GitHub, which lists the downloads. */
  url: string;
}

export function parseVersion(version: string): ParsedVersion | null {
  const match = version.trim().match(VERSION_REGEX);
  if (!match) return null;

  return {
    numbers: [Number(match[1]), Number(match[2]), Number(match[3])],
    prerelease: match[4] ?? null,
  };
}

/**
 * Whether `latest` is a newer version than `current`. A release is newer than a prerelease of the
 * same version, and two prereleases of the same version count as equal. Either version failing to
 * parse counts as not newer, so a malformed tag never prompts an update.
 */
export function isNewerVersion(latest: string, current: string): boolean {
  const a = parseVersion(latest);
  const b = parseVersion(current);
  if (!a || !b) return false;

  for (let i = 0; i < 3; i++) {
    if (a.numbers[i] !== b.numbers[i]) return a.numbers[i] > b.numbers[i];
  }

  return a.prerelease === null && b.prerelease !== null;
}

/** Asks GitHub for Sage's latest release. */
export async function fetchLatestRelease(): Promise<LatestRelease> {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), REQUEST_TIMEOUT_MS);

  try {
    const response = await fetch(sageLatestReleaseApiUrl, {
      headers: { Accept: 'application/vnd.github+json' },
      signal: controller.signal,
    });

    if (!response.ok) {
      throw new Error(`GitHub returned status ${response.status}`);
    }

    const body: unknown = await response.json();
    const tag =
      typeof body === 'object' && body !== null && 'tag_name' in body
        ? body.tag_name
        : undefined;

    if (typeof tag !== 'string' || !parseVersion(tag)) {
      throw new Error(`The latest release has an unexpected tag: ${tag}`);
    }

    // The page is built from the tag rather than taken from the response, so the app only ever
    // opens Sage's own releases page.
    return {
      version: tag.replace(/^v/, ''),
      url: sageReleasePageUrl(tag),
    };
  } finally {
    clearTimeout(timeout);
  }
}
