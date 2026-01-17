import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

type ApiClient = typeof import('./api').api;

let api: ApiClient;

async function loadApi(viteApiUrl: string | undefined): Promise<ApiClient> {
  vi.resetModules();

  const env = (import.meta as any).env ?? ((import.meta as any).env = {});
  if (viteApiUrl === undefined) {
    delete env.VITE_API_URL;
  } else {
    env.VITE_API_URL = viteApiUrl;
  }

  const mod = await import('./api');
  return mod.api;
}

// Mock fetch globally
const mockFetch = vi.fn();
(globalThis as any).fetch = mockFetch;

// Mock localStorage
const localStorageMock = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
};
Object.defineProperty(globalThis, 'localStorage', { value: localStorageMock });

// Helper to create mock response
function createMockResponse(options: {
  ok?: boolean;
  status?: number;
  body?: unknown;
  contentLength?: string | null;
}) {
  const { ok = true, status = 200, body, contentLength } = options;
  const bodyText = body !== undefined ? JSON.stringify(body) : '';
  
  return {
    ok,
    status,
    headers: {
      get: (name: string) => {
        if (name === 'content-length') return contentLength ?? String(bodyText.length);
        return null;
      }
    },
    text: async () => bodyText,
    json: async () => body,
  };
}

describe('API Client', () => {
  beforeEach(async () => {
    mockFetch.mockReset();
    localStorageMock.getItem.mockReset();
    localStorageMock.setItem.mockReset();
    localStorageMock.removeItem.mockReset();

    api = await loadApi('http://localhost:8080');
    api.setToken(null);
  });

  describe('base URL resolution', () => {
    it('defaults to same-origin when VITE_API_URL is unset', async () => {
      vi.resetModules();
      const { resolveApiBase } = await import('./api');

      expect(resolveApiBase(undefined)).toBe('');
      expect(resolveApiBase(null)).toBe('');
    });

    it('trims trailing slashes from VITE_API_URL', async () => {
      vi.resetModules();
      const { resolveApiBase } = await import('./api');

      expect(resolveApiBase('http://localhost:8080/')).toBe('http://localhost:8080');
      expect(resolveApiBase('http://localhost:8080///')).toBe('http://localhost:8080');
    });
  });

  describe('fetch handling', () => {
    it('handles successful response', async () => {
      mockFetch.mockResolvedValueOnce(createMockResponse({ body: { data: 'test' } }));

      const stocks = await api.getStocks();
      
      expect(mockFetch).toHaveBeenCalled();
      expect(stocks).toEqual({ data: 'test' });
    });

    it('handles 204 No Content response', async () => {
      mockFetch.mockResolvedValueOnce(createMockResponse({
        status: 204,
        contentLength: '0',
      }));

      // removeFromWatchlist returns void, should not throw
      await expect(api.removeFromWatchlist('BBCA')).resolves.not.toThrow();
    });

    it('handles empty response body', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        headers: { get: () => null },
        text: async () => '',
      });

      // Should not throw on empty response
      await expect(api.removeFromWatchlist('BBCA')).resolves.not.toThrow();
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
      mockFetch.mockResolvedValueOnce(createMockResponse({
        body: { token: 'new-token', expires_at: 12345 },
      }));

      await api.login('user', 'pass');
      
      expect(localStorageMock.setItem).toHaveBeenCalledWith('token', 'new-token');
    });

    it('clears token on logout', async () => {
      api.setToken('test-token');
      
      mockFetch.mockResolvedValueOnce(createMockResponse({
        body: { success: true },
      }));

      await api.logout();
      
      expect(localStorageMock.removeItem).toHaveBeenCalledWith('token');
    });

    it('clears token on logout even if API returns empty response', async () => {
      api.setToken('test-token');
      
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        headers: { get: () => null },
        text: async () => '',
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
      
      mockFetch.mockResolvedValueOnce(createMockResponse({ body: { stocks: [] } }));

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
      mockFetch.mockResolvedValueOnce(createMockResponse({ body: { stocks: [], count: 0 } }));

      await api.getStocks('banking', 10);
      
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/stocks?sector=banking&limit=10'),
        expect.any(Object)
      );
    });

    it('fetches single stock', async () => {
      mockFetch.mockResolvedValueOnce(createMockResponse({
        body: { symbol: 'BBCA', name: 'Bank Central Asia' },
      }));

      const stock = await api.getStock('BBCA');
      
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/stocks/BBCA'),
        expect.any(Object)
      );
      expect(stock.symbol).toBe('BBCA');
    });

    it('fetches stock prices', async () => {
      mockFetch.mockResolvedValueOnce(createMockResponse({ body: [{ close: 9000 }] }));

      await api.getStockPrices('BBCA', 30);
      
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/stocks/BBCA/prices?days=30'),
        expect.any(Object)
      );
    });

    it('fetches top scores', async () => {
      mockFetch.mockResolvedValueOnce(createMockResponse({
        body: [{ symbol: 'BBCA', composite_score: 85 }],
      }));

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
      mockFetch.mockResolvedValueOnce(createMockResponse({
        body: { id: 1, symbol: 'BBCA' },
      }));

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
      mockFetch.mockResolvedValueOnce(createMockResponse({ body: {} }));

      await api.removeFromWatchlist('BBCA');
      
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/watchlist/BBCA'),
        expect.objectContaining({ method: 'DELETE' })
      );
    });
  });

  describe('fundamentals API', () => {
    it('fetches fundamentals data', async () => {
      mockFetch.mockResolvedValueOnce(createMockResponse({
        body: { symbol: 'BBCA', pe_ratio: 15.5 },
      }));

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
