import { writable } from 'svelte/store';
import { api } from '$lib/api';

interface AuthState {
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
}

function createAuthStore() {
  const { subscribe, set, update } = writable<AuthState>({
    isAuthenticated: false,
    isLoading: true,
    error: null
  });

  return {
    subscribe,

    init() {
      const isAuth = api.isAuthenticated();
      set({ isAuthenticated: isAuth, isLoading: false, error: null });
    },

    async login(username: string, password: string) {
      update((s) => ({ ...s, isLoading: true, error: null }));
      try {
        await api.login(username, password);
        set({ isAuthenticated: true, isLoading: false, error: null });
        return true;
      } catch (e) {
        set({ isAuthenticated: false, isLoading: false, error: (e as Error).message });
        return false;
      }
    },

    async logout() {
      try {
        await api.logout();
      } catch {
        // Even if API fails, clear local auth state
      }
      set({ isAuthenticated: false, isLoading: false, error: null });
    },

    clearError() {
      update((s) => ({ ...s, error: null }));
    }
  };
}

export const auth = createAuthStore();
