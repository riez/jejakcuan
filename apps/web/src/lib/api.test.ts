import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { api } from './api';

// Mock fetch globally
const mockFetch = vi.fn();
global.fetch = mockFetch;

// Mock localStorage
const localStorageMock = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
};
Object.defineProperty(global, 'localStorage', { value: localStorageMock });

describe('API Client', () => {
  beforeEach(() => {
    mockFetch.mockReset();
    localStorageMock.getItem.mockReset();
    localStorageMock.setItem.mockReset();
    localStorageMock.removeItem.mockReset();
    api.setToken(null);
  });

  describe('fetch handling', () => {
    it('handles successful response', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ data: 'test' }),
      });

      const stocks = await api.getStocks();
      
      expect(mockFetch).toHaveBeenCalled();
      expect(stocks).toEqual({ data: 'test' });
    });

    it('handles error response', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 404,
        text: async () => 'Not found',
      });

      await expect(api.getStocks()).rejects.toThrow('Not found');
    });

    it('handles 401 unauthorized and clears token', async () => {
      api.setToken('test-token');
      
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 401,
        text: async () => 'Unauthorized',
      });

      await expect(api.getStocks()).rejects.toThrow('Unauthorized');
      expect(localStorageMock.removeItem).toHaveBeenCalledWith('token');
    });
  });

  describe('authentication', () => {
    it('stores token on successful login', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ token: 'new-token', expires_at: 12345 }),
      });

      await api.login('user', 'pass');
      
      expect(localStorageMock.setItem).toHaveBeenCalledWith('token', 'new-token');
    });

    it('clears token on logout', async () => {
      api.setToken('test-token');
      
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({}),
      });

      await api.logout();
      
      expect(localStorageMock.removeItem).toHaveBeenCalledWith('token');
    });

    it('returns authentication status', () => {
      localStorageMock.getItem.mockReturnValue(null);
      expect(api.isAuthenticated()).toBe(false);
      
      api.setToken('test-token');
      expect(api.isAuthenticated()).toBe(true);
    });

    it('includes authorization header when token is set', async () => {
      api.setToken('test-token');
      
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ stocks: [] }),
      });

      await api.getStocks();
      
      expect(mockFetch).toHaveBeenCalledWith(
        expect.any(String),
        expect.objectContaining({
          headers: expect.objectContaining({
            'Authorization': 'Bearer test-token',
          }),
        })
      );
    });
  });

  describe('stocks API', () => {
    it('fetches stocks with optional filters', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ stocks: [], count: 0 }),
      });

      await api.getStocks('banking', 10);
      
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/stocks?sector=banking&limit=10'),
        expect.any(Object)
      );
    });

    it('fetches single stock', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ symbol: 'BBCA', name: 'Bank Central Asia' }),
      });

      const stock = await api.getStock('BBCA');
      
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/stocks/BBCA'),
        expect.any(Object)
      );
      expect(stock.symbol).toBe('BBCA');
    });

    it('fetches stock prices', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => [{ close: 9000 }],
      });

      await api.getStockPrices('BBCA', 30);
      
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/stocks/BBCA/prices?days=30'),
        expect.any(Object)
      );
    });

    it('fetches top scores', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => [{ symbol: 'BBCA', composite_score: 85 }],
      });

      const scores = await api.getTopScores(5);
      
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/stocks/scores/top?limit=5'),
        expect.any(Object)
      );
      expect(scores[0].composite_score).toBe(85);
    });
  });

  describe('watchlist API', () => {
    it('adds stock to watchlist', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ id: 1, symbol: 'BBCA' }),
      });

      await api.addToWatchlist('BBCA');
      
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/watchlist'),
        expect.objectContaining({
          method: 'POST',
          body: JSON.stringify({ symbol: 'BBCA' }),
        })
      );
    });

    it('removes stock from watchlist', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({}),
      });

      await api.removeFromWatchlist('BBCA');
      
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/watchlist/BBCA'),
        expect.objectContaining({ method: 'DELETE' })
      );
    });
  });

  describe('fundamentals API', () => {
    it('fetches fundamentals data', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ symbol: 'BBCA', pe_ratio: 15.5 }),
      });

      const data = await api.getFundamentals('BBCA');
      
      expect(data?.pe_ratio).toBe(15.5);
    });

    it('returns null for 404 (no data available)', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 404,
        text: async () => 'HTTP 404',
      });

      const data = await api.getFundamentals('UNKNOWN');
      
      expect(data).toBeNull();
    });
  });
});
