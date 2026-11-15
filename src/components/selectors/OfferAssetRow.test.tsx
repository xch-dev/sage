// @vitest-environment jsdom

import { i18n } from '@lingui/core';
import { I18nProvider } from '@lingui/react';
import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, beforeAll, describe, expect, it } from 'vitest';
import { NftLookup } from '@/hooks/useRequestedNft';
import { RequestedNftPreview } from './OfferAssetRow';

beforeAll(() => {
  i18n.loadAndActivate({ locale: 'en', messages: {} });
});

afterEach(cleanup);

function renderPreview(lookup: NftLookup) {
  return render(
    <I18nProvider i18n={i18n}>
      <RequestedNftPreview lookup={lookup} />
    </I18nProvider>,
  );
}

describe('RequestedNftPreview', () => {
  it('shows nothing before an ID is typed', () => {
    const { container } = renderPreview({ status: 'idle' });
    expect(container.textContent).toBe('');
  });

  it('says when the ID is not valid', () => {
    renderPreview({ status: 'invalid' });
    expect(screen.getByText("This isn't a valid NFT ID.")).toBeTruthy();
  });

  it('says when the NFT could not be found', () => {
    renderPreview({ status: 'missing' });
    expect(screen.getByText(/Couldn't find this NFT/)).toBeTruthy();
  });

  it('shows the name and marks a mutable NFT', () => {
    renderPreview({ status: 'found', name: 'Cool NFT', kind: 'any' });
    expect(screen.getByText('Cool NFT')).toBeTruthy();
    expect(screen.getByLabelText('Mutable NFT')).toBeTruthy();
  });

  it("doesn't call a found NFT unnamed before its name is known", () => {
    renderPreview({ status: 'found', name: null, kind: 'default' });
    expect(screen.getByText('NFT found')).toBeTruthy();
    expect(screen.queryByLabelText('Mutable NFT')).toBeNull();
  });
});
