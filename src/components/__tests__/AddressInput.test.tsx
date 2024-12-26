import { describe, expect, it, vi } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import { AddressInput } from '../AddressInput';
import * as namesdao from '@/utils/namesdao';

// Spy on the actual namesdao functions instead of completely mocking them
vi.spyOn(namesdao, 'isValidXchName');
vi.spyOn(namesdao, 'resolveXchName');

describe('AddressInput', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    // Default mock implementation for successful case
    (namesdao.resolveXchName as any).mockImplementation(async (name: string) => {
      if (name === 'namesdao.xch') {
        return 'xch1jhye8dmkhree0zr8t09rlzm9cc82mhuqtp5tlmsj4kuqvs69s2wsl90su4';
      }
      return null;
    });
  });

  it('renders input field', () => {
    render(<AddressInput />);
    expect(screen.getByPlaceholderText('Enter address or .xch name')).toBeInTheDocument();
  });

  it('resolves valid .xch name', async () => {
    const onResolvedAddress = vi.fn();
    render(<AddressInput value="namesdao.xch" onResolvedAddress={onResolvedAddress} />);

    await waitFor(() => {
      expect(screen.getByText(/Resolves to:/)).toBeInTheDocument();
      expect(onResolvedAddress).toHaveBeenCalledWith(
        'xch1jhye8dmkhree0zr8t09rlzm9cc82mhuqtp5tlmsj4kuqvs69s2wsl90su4'
      );
    });
  });

  it('shows loading state while resolving', async () => {
    render(<AddressInput value="namesdao.xch" />);
    
    // Should show loading state immediately
    expect(screen.getByText('Resolving name...')).toBeInTheDocument();
    
    // Should show resolved address after loading
    await waitFor(() => {
      expect(screen.getByText(/Resolves to:/)).toBeInTheDocument();
    });
  });

  it('handles invalid .xch names', async () => {
    render(<AddressInput value="-----.xch" />);
    
    await waitFor(() => {
      expect(screen.getByText('Invalid .xch name')).toBeInTheDocument();
    });
  });

  it('handles network errors during name resolution', async () => {
    // Mock a network error for this specific test
    (namesdao.resolveXchName as any).mockImplementationOnce(async () => {
      throw new Error('Network error');
    });

    render(<AddressInput value="namesdao.xch" />);
    
    await waitFor(() => {
      expect(screen.getByText('Failed to resolve name')).toBeInTheDocument();
    });
  });

  it('clears resolution state for non-.xch input', async () => {
    const { rerender } = render(<AddressInput value="namesdao.xch" />);
    
    // Wait for initial resolution
    await waitFor(() => {
      expect(screen.getByText(/Resolves to:/)).toBeInTheDocument();
    });

    // Change to non-.xch input
    rerender(<AddressInput value="xch1..." />);
    
    await waitFor(() => {
      expect(screen.queryByText(/Resolves to:/)).not.toBeInTheDocument();
      expect(screen.queryByText('Resolving name...')).not.toBeInTheDocument();
      expect(screen.queryByText('Invalid .xch name')).not.toBeInTheDocument();
    });
  });
});
