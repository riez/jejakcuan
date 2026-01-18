export function resolveApiBase(viteApiUrl: string | undefined | null): string {
  return (viteApiUrl ?? '').replace(/\/+$/, '');
}

const API_BASE = resolveApiBase((import.meta as any).env?.VITE_API_URL);

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

interface RecomputeScoresResponse {
  computed: number;
  skipped: number;
  errors: number;
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

interface StockFreshness {
  symbol: string;
  prices_as_of: string | null;
  broker_flow_as_of: string | null;
  financials_as_of: string | null;
  scores_as_of: string | null;
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

// Admin Types (Legacy)
interface DataSourceStatus {
  id: string;
  name: string;
  source_type: string;
  last_update: string | null;
  record_count: number;
  status: string;
  freshness_hours: number | null;
  can_refresh: boolean;
}

interface DataSummary {
  total_stocks: number;
  stocks_with_prices: number;
  stocks_with_scores: number;
  stocks_with_broker_data: number;
  oldest_price_data: string | null;
  newest_price_data: string | null;
}

interface DataStatusResponse {
  timestamp: string;
  overall_status: string;
  sources: DataSourceStatus[];
  summary: DataSummary;
}

interface RefreshResponse {
  source_id: string;
  status: string;
  message: string;
  started_at: string;
}

// Granular Data Source Types
type DataSourceCategory = 'broker' | 'prices' | 'fundamentals' | 'scores';
type SourceType = 'python_scraper' | 'rust_client' | 'computed';
type DataSourceState = 'fresh' | 'stale' | 'outdated' | 'no_data' | 'not_configured' | 'running' | 'error';

interface ConfigFieldStatus {
  name: string;
  description: string;
  env_var: string;
  required: boolean;
  is_set: boolean;
}

interface ConfigStatus {
  is_configured: boolean;
  missing_fields: string[];
  config_fields: ConfigFieldStatus[];
}

interface GranularDataSource {
  id: string;
  name: string;
  category: DataSourceCategory;
  category_name: string;
  source_type: SourceType;
  description: string;
  status: DataSourceState;
  config_status: ConfigStatus;
  last_update: string | null;
  record_count: number;
  freshness_hours: number | null;
  can_trigger: boolean;
  trigger_command: string | null;
}

interface CategorySummary {
  category: string;
  display_name: string;
  total: number;
  fresh: number;
  stale: number;
  not_configured: number;
}

interface DataSourcesSummary {
  total_sources: number;
  configured_sources: number;
  fresh_sources: number;
  stale_sources: number;
  categories: CategorySummary[];
}

interface DataSourcesResponse {
  timestamp: string;
  overall_status: string;
  sources: GranularDataSource[];
  by_category: Record<string, GranularDataSource[]>;
  summary: DataSourcesSummary;
}

type JobStatus = 'pending' | 'running' | 'completed' | 'failed';

interface Job {
  id: string;
  source_id: string;
  source_name: string;
  command: string;
  status: JobStatus;
  message: string | null;
  output: string | null;
  started_at: string;
  completed_at: string | null;
  duration_secs: number | null;
}

interface JobsListResponse {
  jobs: Job[];
  count: number;
}

interface TriggerResponse {
  source_id: string;
  status: string;
  message: string;
  command: string | null;
  started_at: string;
  job_id?: string;
  job?: Job;
}

interface RefreshStockResponse {
  symbol: string;
  jobs: Job[];
  message: string;
}

interface RefreshSourceResponse {
  symbol: string;
  source_type: string;
  job: Job;
}

type StockSourceType = 'price' | 'broker' | 'fundamental';

interface SkippedSource {
  source_id: string;
  reason: string;
}

interface CategoryTriggerResponse {
  category: string;
  triggered: TriggerResponse[];
  skipped: SkippedSource[];
}

interface ConfigResponse {
  source_id: string;
  source_name: string;
  fields: ConfigFieldStatus[];
  is_configured: boolean;
}

// Analysis Types
interface BrokerInfo {
  code: string;
  name: string | null;
  avg_price: number;
  category: string;
  buy_volume: number;
  sell_volume: number;
  net_volume: number;
  buy_value: number;
  sell_value: number;
  net_value: number;
}

interface PriceRange {
  low: number;
  high: number;
}

interface BrokerSummaryResponse {
  big_buyers: BrokerInfo[];
  big_sellers: BrokerInfo[];
  net_status: string;
  price_range: PriceRange;
  foreign_net: number;
  domestic_net: number;
}

interface IchimokuInfo {
  position: string;
  cloud_range: PriceRange;
}

interface TASummary {
  sell: number;
  neutral: number;
  buy: number;
}

interface BollingerResponse {
  upper: number;
  middle: number;
  lower: number;
}

interface TechnicalResponse {
  last_price: number;
  rsi: number;
  rsi_signal: string;
  macd: number;
  macd_signal: string;
  macd_histogram: number;
  bollinger: BollingerResponse;
  ichimoku: IchimokuInfo;
  support: number[];
  resistance: number[];
  summary: TASummary;
}

interface ValuationResponse {
  per_value: number;
  forward_eps: number;
  pbv_value: number;
  book_value: number;
  ev_ebitda_value: number;
  fair_price_range: PriceRange;
  bull_case: PriceRange;
}

interface StrategyResponse {
  traders: string;
  investors: string;
  value_investors: string;
}

interface ConclusionResponse {
  strengths: string[];
  weaknesses: string[];
  strategy: StrategyResponse;
}

interface FullAnalysisResponse {
  symbol: string;
  name: string;
  sector: string | null;
  broker_summary: BrokerSummaryResponse | null;
  technical: TechnicalResponse | null;
  valuation: ValuationResponse | null;
  conclusion: ConclusionResponse | null;
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
        if (typeof window !== 'undefined' && !window.location.pathname.includes('/login')) {
          window.location.href = '/login';
        }
        throw new Error('Unauthorized');
      }
      const error = await response.text();
      throw new Error(error || `HTTP ${response.status}`);
    }

    // Handle empty responses (204 No Content or empty body)
    const contentLength = response.headers.get('content-length');
    if (response.status === 204 || contentLength === '0') {
      return {} as T;
    }

    // Try to parse JSON, fallback to empty object for empty responses
    const text = await response.text();
    if (!text || text.trim() === '') {
      return {} as T;
    }

    return JSON.parse(text) as T;
  }

  private async fetchWithTimeout<T>(
    path: string,
    options: RequestInit = {},
    timeoutMs: number = 20000
  ): Promise<T> {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), timeoutMs);
    try {
      return await this.fetch<T>(path, { ...options, signal: controller.signal });
    } catch (error) {
      const err = error as any;
      if (err?.name === 'AbortError') {
        throw new Error('Request timeout');
      }
      throw error;
    } finally {
      clearTimeout(timeoutId);
    }
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
    const raw = await this.fetch<any[]>(`/api/stocks/${symbol}/prices${params}`);
    return raw.map((p) => ({
      time: String(p.time),
      symbol: String(p.symbol ?? symbol).toUpperCase(),
      open: Number(p.open),
      high: Number(p.high),
      low: Number(p.low),
      close: Number(p.close),
      volume: Number(p.volume),
    }));
  }

  async getStockScore(symbol: string): Promise<StockScore | null> {
    return this.fetch(`/api/stocks/${symbol}/score`);
  }

  async getStockFreshness(symbol: string): Promise<StockFreshness> {
    return this.fetch(`/api/stocks/${symbol}/freshness`);
  }

  async refreshStockData(symbol: string): Promise<RefreshStockResponse> {
    return this.fetch(`/api/stocks/${symbol}/refresh`, { method: 'POST' });
  }

  async refreshStockSource(symbol: string, sourceType: StockSourceType): Promise<RefreshSourceResponse> {
    return this.fetch(`/api/stocks/${symbol}/refresh/${sourceType}`, { method: 'POST' });
  }

  async getTopScores(limit?: number): Promise<StockScore[]> {
    const params = limit ? `?limit=${limit}` : '';
    return this.fetch(`/api/stocks/scores/top${params}`);
  }

  async recomputeScores(): Promise<RecomputeScoresResponse> {
    return this.fetch('/api/stocks/scores/recompute', { method: 'POST' });
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

  // Analysis
  async getFullAnalysis(symbol: string, days?: number): Promise<FullAnalysisResponse | null> {
    try {
      const params = days ? `?days=${days}` : '';
      return await this.fetchWithTimeout<FullAnalysisResponse>(`/api/analysis/${symbol}/analysis${params}`);
    } catch (error) {
      if (error instanceof Error && (error.message.includes('404') || error.message.includes('400'))) {
        return null;
      }
      throw error;
    }
  }

  async getTechnicals(symbol: string, days?: number): Promise<TechnicalResponse | null> {
    try {
      const params = days ? `?days=${days}` : '';
      return await this.fetchWithTimeout<TechnicalResponse>(`/api/analysis/${symbol}/technicals${params}`);
    } catch (error) {
      if (error instanceof Error && (error.message.includes('404') || error.message.includes('400'))) {
        return null;
      }
      throw error;
    }
  }

  async getBrokerFlow(symbol: string, days?: number): Promise<BrokerSummaryResponse | null> {
    try {
      const params = days ? `?days=${days}` : '';
      return await this.fetchWithTimeout<BrokerSummaryResponse>(`/api/analysis/${symbol}/broker-flow${params}`);
    } catch (error) {
      if (error instanceof Error && (error.message.includes('404') || error.message.includes('400'))) {
        return null;
      }
      throw error;
    }
  }

  async getDataStatus(): Promise<DataStatusResponse> {
    return this.fetch('/api/admin/data-status');
  }

  async getSourceStatus(sourceId: string): Promise<DataSourceStatus> {
    return this.fetch(`/api/admin/data-status/${sourceId}`);
  }

  async refreshSource(sourceId: string): Promise<RefreshResponse> {
    return this.fetch(`/api/admin/data-status/${sourceId}/refresh`, { method: 'POST' });
  }

  async getDataSources(): Promise<DataSourcesResponse> {
    return this.fetch('/api/admin/data-sources');
  }

  async getDataSource(sourceId: string): Promise<GranularDataSource> {
    return this.fetch(`/api/admin/data-sources/${sourceId}`);
  }

  async triggerDataSource(sourceId: string): Promise<TriggerResponse> {
    return this.fetch(`/api/admin/data-sources/${sourceId}/trigger`, { method: 'POST' });
  }

  async triggerCategory(category: string): Promise<CategoryTriggerResponse> {
    return this.fetch(`/api/admin/data-sources/category/${category}/trigger`, { method: 'POST' });
  }

  async getSourceConfig(sourceId: string): Promise<ConfigResponse> {
    return this.fetch(`/api/admin/data-sources/${sourceId}/config`);
  }

  async getJobs(): Promise<JobsListResponse> {
    return this.fetch('/api/admin/jobs');
  }

  async getJob(jobId: string): Promise<Job> {
    return this.fetch(`/api/admin/jobs/${jobId}`);
  }

  async getSourceJobs(sourceId: string): Promise<JobsListResponse> {
    return this.fetch(`/api/admin/jobs/source/${sourceId}`);
  }
}

export const api = new ApiClient();
export type { 
  Stock, 
  StockScore, 
  StockPrice, 
  StockFreshness,
  WatchlistItem, 
  LoginResponse, 
  FundamentalData,
  FullAnalysisResponse,
  TechnicalResponse,
  BrokerSummaryResponse,
  ValuationResponse,
  ConclusionResponse,
  BrokerInfo,
  PriceRange,
  IchimokuInfo,
  TASummary,
  BollingerResponse,
  StrategyResponse,
  RecomputeScoresResponse,
  DataStatusResponse,
  DataSourceStatus,
  DataSummary,
  RefreshResponse,
  DataSourceCategory,
  SourceType,
  DataSourceState,
  ConfigFieldStatus,
  ConfigStatus,
  GranularDataSource,
  CategorySummary,
  DataSourcesSummary,
  DataSourcesResponse,
  TriggerResponse,
  SkippedSource,
  CategoryTriggerResponse,
  ConfigResponse,
  Job,
  JobStatus,
  JobsListResponse,
  RefreshStockResponse,
  RefreshSourceResponse,
  StockSourceType
};
