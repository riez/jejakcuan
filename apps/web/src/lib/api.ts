const API_BASE = import.meta.env.VITE_API_URL || 'http://localhost:8080';

interface LoginRequest {
  username: string;
  password: string;
}

interface LoginResponse {
  token: string;
  expires_at: number;
}

interface Stock {
  id: number;
  symbol: string;
  name: string;
  sector: string | null;
  subsector: string | null;
  is_active: boolean;
}

interface StockScore {
  time: string;
  symbol: string;
  composite_score: number;
  technical_score: number;
  fundamental_score: number;
  sentiment_score: number;
  ml_score: number;
}

interface StockPrice {
  time: string;
  symbol: string;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
}

interface WatchlistItem {
  id: number;
  symbol: string;
  sort_order: number;
  notes: string | null;
  added_at: string;
}

interface FundamentalData {
  symbol: string;
  pe_ratio: number | null;
  pb_ratio: number | null;
  ps_ratio: number | null;
  ev_ebitda: number | null;
  roe: number | null;
  roa: number | null;
  profit_margin: number | null;
  debt_to_equity: number | null;
  current_ratio: number | null;
  dcf_intrinsic_value: number | null;
  dcf_margin_of_safety: number | null;
  sector_avg_pe: number | null;
  sector_avg_pb: number | null;
}

class ApiClient {
  private token: string | null = null;

  setToken(token: string | null) {
    this.token = token;
    if (token) {
      if (typeof localStorage !== 'undefined') {
        localStorage.setItem('token', token);
      }
    } else {
      if (typeof localStorage !== 'undefined') {
        localStorage.removeItem('token');
      }
    }
  }

  getToken(): string | null {
    if (this.token) return this.token;
    if (typeof localStorage !== 'undefined') {
      return localStorage.getItem('token');
    }
    return null;
  }

  private async fetch<T>(path: string, options: RequestInit = {}): Promise<T> {
    const token = this.getToken();
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...((options.headers as Record<string, string>) || {})
    };

    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    const response = await fetch(`${API_BASE}${path}`, {
      ...options,
      headers,
      credentials: 'include'
    });

    if (!response.ok) {
      if (response.status === 401) {
        this.setToken(null);
        throw new Error('Unauthorized');
      }
      const error = await response.text();
      throw new Error(error || `HTTP ${response.status}`);
    }

    return response.json();
  }

  // Auth
  async login(username: string, password: string): Promise<LoginResponse> {
    const response = await this.fetch<LoginResponse>('/api/auth/login', {
      method: 'POST',
      body: JSON.stringify({ username, password })
    });
    this.setToken(response.token);
    return response;
  }

  async logout(): Promise<void> {
    try {
      await this.fetch('/api/auth/logout', { method: 'POST' });
    } finally {
      this.setToken(null);
    }
  }

  isAuthenticated(): boolean {
    return !!this.getToken();
  }

  // Stocks
  async getStocks(sector?: string, limit?: number): Promise<{ stocks: Stock[]; count: number }> {
    const params = new URLSearchParams();
    if (sector) params.set('sector', sector);
    if (limit) params.set('limit', limit.toString());
    const query = params.toString();
    return this.fetch(`/api/stocks${query ? `?${query}` : ''}`);
  }

  async getStock(symbol: string): Promise<Stock> {
    return this.fetch(`/api/stocks/${symbol}`);
  }

  async getStockPrices(symbol: string, days?: number): Promise<StockPrice[]> {
    const params = days ? `?days=${days}` : '';
    return this.fetch(`/api/stocks/${symbol}/prices${params}`);
  }

  async getStockScore(symbol: string): Promise<StockScore | null> {
    return this.fetch(`/api/stocks/${symbol}/score`);
  }

  async getTopScores(limit?: number): Promise<StockScore[]> {
    const params = limit ? `?limit=${limit}` : '';
    return this.fetch(`/api/stocks/scores/top${params}`);
  }

  // Watchlist
  async getWatchlist(): Promise<WatchlistItem[]> {
    return this.fetch('/api/watchlist');
  }

  async addToWatchlist(symbol: string): Promise<WatchlistItem> {
    return this.fetch('/api/watchlist', {
      method: 'POST',
      body: JSON.stringify({ symbol })
    });
  }

  async removeFromWatchlist(symbol: string): Promise<void> {
    await this.fetch(`/api/watchlist/${symbol}`, { method: 'DELETE' });
  }

  // Fundamentals
  async getFundamentals(symbol: string): Promise<FundamentalData | null> {
    try {
      return await this.fetch<FundamentalData>(`/api/stocks/${symbol}/fundamentals`);
    } catch (error) {
      // Return null for 404 (no data available)
      if (error instanceof Error && error.message.includes('404')) {
        return null;
      }
      throw error;
    }
  }
}

export const api = new ApiClient();
export type { Stock, StockScore, StockPrice, WatchlistItem, LoginResponse, FundamentalData };
