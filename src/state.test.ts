import { vi, describe, expect, it, beforeEach } from 'vitest';

// Mock bindings before any imports that use them
vi.mock('@/bindings', () => ({
  commands: {
    getKey: vi.fn(),
    getSyncStatus: vi.fn(),
    login: vi.fn(),
    logout: vi.fn(),
  },
  events: {
    syncEvent: {
      listen: vi.fn(async () => vi.fn()),
    },
  },
}));

import { commands } from '@/bindings';
import {
  useWalletState,
  useOfferState,
  useNavigationStore,
  clearState,
  loginAndUpdateState,
  defaultState,
} from './state';

const mockCommands = vi.mocked(commands);

describe('useWalletState', () => {
  beforeEach(() => {
    clearState();
  });

  it('has correct default state', () => {
    const state = useWalletState.getState();
    expect(state.sync.balance).toBe('0');
    expect(state.sync.unit.ticker).toBe('XCH');
    expect(state.sync.unit.precision).toBe(12);
    expect(state.sync.receive_address).toBe('Unknown');
    expect(state.sync.total_coins).toBe(0);
    expect(state.sync.synced_coins).toBe(0);
  });

  it('can be updated via setState', () => {
    useWalletState.setState({
      sync: { ...defaultState().sync, balance: '1000' },
    });
    expect(useWalletState.getState().sync.balance).toBe('1000');
  });
});

describe('useOfferState', () => {
  beforeEach(() => {
    clearState();
  });

  it('defaults to null', () => {
    expect(useOfferState.getState()).toBeNull();
  });

  it('can be set to an offer', () => {
    useOfferState.setState({
      offered: { tokens: [], nfts: [], options: [] },
      requested: { tokens: [], nfts: [], options: [] },
      fee: '0',
      expiration: null,
    });
    expect(useOfferState.getState()).not.toBeNull();
    expect(useOfferState.getState()?.fee).toBe('0');
  });
});

describe('useNavigationStore', () => {
  it('starts with empty returnValues', () => {
    const state = useNavigationStore.getState();
    expect(state.returnValues).toEqual({});
  });

  it('setReturnValue stores and retrieves values', () => {
    useNavigationStore
      .getState()
      .setReturnValue('page1', { status: 'success', data: 'test' });

    const state = useNavigationStore.getState();
    expect(state.returnValues['page1']).toEqual({
      status: 'success',
      data: 'test',
    });
  });

  it('setReturnValue preserves existing values', () => {
    useNavigationStore
      .getState()
      .setReturnValue('page1', { status: 'success' });
    useNavigationStore
      .getState()
      .setReturnValue('page2', { status: 'cancelled' });

    const state = useNavigationStore.getState();
    expect(state.returnValues['page1']?.status).toBe('success');
    expect(state.returnValues['page2']?.status).toBe('cancelled');
  });
});

describe('clearState', () => {
  it('resets wallet state to defaults', () => {
    useWalletState.setState({
      sync: { ...defaultState().sync, balance: '999' },
    });
    clearState();
    expect(useWalletState.getState().sync.balance).toBe('0');
  });

  it('resets offer state to null', () => {
    useOfferState.setState({
      offered: { tokens: [], nfts: [], options: [] },
      requested: { tokens: [], nfts: [], options: [] },
      fee: '100',
      expiration: null,
    });
    clearState();
    expect(useOfferState.getState()).toBeNull();
  });
});

describe('loginAndUpdateState', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    clearState();
  });

  it('calls login then fetches sync status', async () => {
    const syncResponse = {
      ...defaultState().sync,
      balance: '42000',
      receive_address: 'xch1abc',
    };

    mockCommands.login.mockResolvedValue(undefined as never);
    mockCommands.getKey.mockResolvedValue({ key: { fingerprint: 123 } } as never);
    mockCommands.getSyncStatus.mockResolvedValue(syncResponse as never);

    await loginAndUpdateState(123);

    expect(mockCommands.login).toHaveBeenCalledWith({ fingerprint: 123 });
    expect(mockCommands.getKey).toHaveBeenCalled();
    expect(mockCommands.getSyncStatus).toHaveBeenCalled();
  });

  it('calls onError when login fails', async () => {
    const mockError = { kind: 'test', reason: 'fail' };
    mockCommands.login.mockRejectedValue(mockError);

    const onError = vi.fn();

    await expect(loginAndUpdateState(123, onError)).rejects.toBe(mockError);
    expect(onError).toHaveBeenCalledWith(mockError);
  });

  it('throws without onError when login fails', async () => {
    const mockError = new Error('login failed');
    mockCommands.login.mockRejectedValue(mockError);

    await expect(loginAndUpdateState(123)).rejects.toThrow('login failed');
  });
});
