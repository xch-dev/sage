// @vitest-environment jsdom

import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import MnemonicDisplay from './MnemonicDisplay';

afterEach(cleanup);

function mnemonic(length: number, prefix = 'word') {
  return Array.from({ length }, (_, index) => `${prefix}-${index}`).join(' ');
}

function overlappingMnemonic(generation: number) {
  const words = Array.from(
    { length: 24 },
    (_, index) => `word-${(generation + index) % 32}`,
  );
  words[23] = words[0];
  return words.join(' ');
}

describe('MnemonicDisplay', () => {
  it('renders every position when words repeat', () => {
    const words = mnemonic(24).split(' ');
    words[23] = words[0];

    render(<MnemonicDisplay mnemonic={words.join(' ')} />);

    expect(screen.getAllByRole('listitem')).toHaveLength(24);
  });

  it('replaces the entire phrase without retaining old words', () => {
    const { rerender } = render(
      <MnemonicDisplay mnemonic={mnemonic(24, 'old')} />,
    );

    rerender(<MnemonicDisplay mnemonic={mnemonic(24, 'new')} />);

    const displayedWords = screen
      .getAllByRole('listitem')
      .map((item) => item.textContent);
    expect(displayedWords).toEqual(mnemonic(24, 'new').split(' '));
  });

  it('stays at exactly 24 words through many generations', () => {
    const { rerender } = render(
      <MnemonicDisplay mnemonic={overlappingMnemonic(0)} />,
    );

    for (let generation = 1; generation <= 100; generation++) {
      const currentMnemonic = overlappingMnemonic(generation);
      rerender(<MnemonicDisplay mnemonic={currentMnemonic} />);

      const displayedWords = screen
        .getAllByRole('listitem')
        .map((item) => item.textContent);
      expect(displayedWords).toEqual(currentMnemonic.split(' '));
    }
  });
});
