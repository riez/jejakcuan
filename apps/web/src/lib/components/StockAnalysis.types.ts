/**
 * Types for Stock Analysis Component
 * Used for broker summary, technical analysis, valuation estimates, and overall conclusions
 */

export interface BrokerSummary {
  bigBuyers: BrokerInfo[];
  bigSellers: BrokerInfo[];
  netStatus: 'accumulation' | 'distribution' | 'balanced';
  priceRange: { low: number; high: number };
  foreignNet?: number;
  domesticNet?: number;
}

export interface BrokerInfo {
  code: string;
  name?: string | null;
  category?: string;
  avgPrice: number;
  buyVolume?: number;
  sellVolume?: number;
  netVolume?: number;
  buyValue?: number;
  sellValue?: number;
  netValue?: number;
}

export interface TechnicalAnalysis {
  lastPrice: number;
  rsi: number;
  rsiSignal: 'oversold' | 'neutral' | 'overbought';
  macd: number;
  macdSignal: 'positive' | 'negative';
  ichimoku: {
    position: 'above' | 'in' | 'below';
    cloudRange: { low: number; high: number };
  };
  support: number[];
  resistance: number[];
  summary: { sell: number; neutral: number; buy: number };
}

export interface ValuationEstimate {
  perValue: number;
  forwardEps: number;
  pbvValue: number;
  bookValue: number;
  evEbitdaValue: number;
  fairPriceRange: { low: number; high: number };
  bullCase: { low: number; high: number };
}

export interface OverallConclusion {
  strengths: string[];
  weaknesses: string[];
  strategy: { traders: string; investors: string; valueInvestors: string };
}
