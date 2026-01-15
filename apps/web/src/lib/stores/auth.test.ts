import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';
import { auth } from './auth';
import { api } from '$lib/api';

// Mock the API module
vi.mock('$lib/api', () => ({
  api: {
    isAuthenticated: vi.fn(),
    login: vi.fn(),
    logout: vi.fn(),
    setToken: vi.fn(),
  },
}));

describe('Auth Store', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('init', () => {
    it('sets isAuthenticated based on API status', () => {
      vi.mocked(api.isAuthenticated).mockReturnValue(true);
      
      auth.init();
      
      const state = get(auth);
      expect(state.isAuthenticated).toBe(true);
      expect(state.isLoading).toBe(false);
      expect(state.error).toBe(null);
    });

    it('sets isAuthenticated to false when not logged in', () => {
      vi.mocked(api.isAuthenticated).mockReturnValue(false);
      
      auth.init();
      
      const state = get(auth);
      expect(state.isAuthenticated).toBe(false);
      expect(state.isLoading).toBe(false);
    });
  });

  describe('login', () => {
    it('returns true and sets authenticated on successful login', async () => {
      vi.mocked(api.login).mockResolvedValue({ token: 'test-token', expires_at: 12345 });
      
      const result = await auth.login('user', 'pass');
      
      expect(result).toBe(true);
      const state = get(auth);
      expect(state.isAuthenticated).toBe(true);
      expect(state.isLoading).toBe(false);
      expect(state.error).toBe(null);
    });

    it('returns false and sets error on failed login', async () => {
      vi.mocked(api.login).mockRejectedValue(new Error('Invalid credentials'));
      
      const result = await auth.login('user', 'wrong');
      
      expect(result).toBe(false);
      const state = get(auth);
      expect(state.isAuthenticated).toBe(false);
      expect(state.isLoading).toBe(false);
      expect(state.error).toBe('Invalid credentials');
    });

    it('sets loading state during login attempt', async () => {
      let loadingDuringRequest = false;
      
      vi.mocked(api.login).mockImplementation(async () => {
        loadingDuringRequest = get(auth).isLoading;
        return { token: 'test-token', expires_at: 12345 };
      });
      
      await auth.login('user', 'pass');
      
      expect(loadingDuringRequest).toBe(true);
    });
  });

  describe('logout', () => {
    it('clears authentication state', async () => {
      vi.mocked(api.logout).mockResolvedValue();
      
      await auth.logout();
      
      const state = get(auth);
      expect(state.isAuthenticated).toBe(false);
      expect(state.isLoading).toBe(false);
      expect(state.error).toBe(null);
      expect(api.logout).toHaveBeenCalled();
    });

    it('handles logout error gracefully', async () => {
      vi.mocked(api.logout).mockRejectedValue(new Error('Network error'));
      
      // Should not throw
      await auth.logout();
      
      // Should still clear auth state even if API fails
      const state = get(auth);
      expect(state.isAuthenticated).toBe(false);
    });
  });

  describe('clearError', () => {
    it('clears the error state', async () => {
      // First, create an error state
      vi.mocked(api.login).mockRejectedValue(new Error('Test error'));
      await auth.login('user', 'wrong');
      
      // Verify error exists
      expect(get(auth).error).toBe('Test error');
      
      // Clear error
      auth.clearError();
      
      expect(get(auth).error).toBe(null);
    });
  });
});
