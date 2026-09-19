// @vitest-environment jsdom

import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import MnemonicDisplay from './MnemonicDisplay';

afterEach(cleanup);

function mnemonic(length: number, prefix = 'word') {
  return Array.from({ length }, (_, index) => `${prefix}-${index}`).join(' ');
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
      <MnemonicDisplay key='first-generation' mnemonic={mnemonic(24, 'old')} />,
    );

    rerender(
      <MnemonicDisplay
        key='second-generation'
        mnemonic={mnemonic(24, 'new')}
      />,
    );

    const displayedWords = screen
      .getAllByRole('listitem')
      .map((item) => item.textContent);
    expect(displayedWords).toEqual(mnemonic(24, 'new').split(' '));
  });
});
