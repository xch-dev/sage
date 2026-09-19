// @vitest-environment jsdom

import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import MnemonicDisplay, { normalizeMnemonic } from './MnemonicDisplay';

afterEach(cleanup);

function mnemonic(length: number, prefix = 'word') {
  return Array.from({ length }, (_, index) => `${prefix}-${index}`).join(' ');
}

describe('normalizeMnemonic', () => {
  it('normalizes whitespace only when the word count is exact', () => {
    const words = mnemonic(24).split(' ');
    const value = `  ${words.slice(0, 12).join('  ')}\n${words
      .slice(12)
      .join('\t')}  `;

    expect(normalizeMnemonic(value, 24)).toBe(words.join(' '));
    expect(normalizeMnemonic(`${value} extra`, 24)).toBeNull();
    expect(normalizeMnemonic(mnemonic(23), 24)).toBeNull();
  });
});

describe('MnemonicDisplay', () => {
  it('renders every position when words repeat', () => {
    const words = mnemonic(24).split(' ');
    words[23] = words[0];

    render(
      <MnemonicDisplay mnemonic={words.join(' ')} expectedWordCount={24} />,
    );

    expect(screen.getAllByRole('listitem')).toHaveLength(24);
  });

  it('replaces the entire phrase without retaining old words', () => {
    const { rerender } = render(
      <MnemonicDisplay
        key='first-generation'
        mnemonic={mnemonic(24, 'old')}
        expectedWordCount={24}
      />,
    );

    rerender(
      <MnemonicDisplay
        key='second-generation'
        mnemonic={mnemonic(24, 'new')}
        expectedWordCount={24}
      />,
    );

    const displayedWords = screen
      .getAllByRole('listitem')
      .map((item) => item.textContent);
    expect(displayedWords).toEqual(mnemonic(24, 'new').split(' '));
  });

  it('renders no partial or oversized phrase', () => {
    const { rerender } = render(
      <MnemonicDisplay mnemonic={mnemonic(25)} expectedWordCount={24} />,
    );

    expect(screen.queryAllByRole('listitem')).toHaveLength(0);

    rerender(
      <MnemonicDisplay mnemonic={mnemonic(23)} expectedWordCount={24} />,
    );
    expect(screen.queryAllByRole('listitem')).toHaveLength(0);
  });
});
