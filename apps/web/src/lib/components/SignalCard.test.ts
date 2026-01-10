import { render, screen } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import SignalCard from './SignalCard.svelte';

describe('SignalCard', () => {
  const mockSignal = {
    id: '1',
    symbol: 'BBCA',
    stockName: 'Bank Central Asia',
    type: 'buy' as const,
    strength: 'strong' as const,
    score: 82,
    reason: 'Strong accumulation pattern',
    timestamp: new Date('2024-01-15T10:00:00'),
    priceAtSignal: 9250,
    targetPrice: 10000,
    stopLoss: 8800,
    indicators: ['EMA Bullish', 'RSI Oversold'],
  };

  it('renders signal symbol and type', () => {
    render(SignalCard, { props: { signal: mockSignal } });
    
    expect(screen.getByText('BBCA')).toBeInTheDocument();
    expect(screen.getByText('BUY')).toBeInTheDocument();
  });

  it('displays score with appropriate color', () => {
    render(SignalCard, { props: { signal: mockSignal } });
    
    const scoreElement = screen.getByText('82');
    expect(scoreElement).toBeInTheDocument();
    expect(scoreElement).toHaveClass('text-green-500');
  });

  it('shows stock name', () => {
    render(SignalCard, { props: { signal: mockSignal } });
    
    expect(screen.getByText('Bank Central Asia')).toBeInTheDocument();
  });

  it('shows price targets', () => {
    render(SignalCard, { props: { signal: mockSignal } });
    
    // Entry price
    expect(screen.getByText('9,250')).toBeInTheDocument();
    // Target price
    expect(screen.getByText('10,000')).toBeInTheDocument();
    // Stop loss
    expect(screen.getByText('8,800')).toBeInTheDocument();
  });

  it('renders indicators as badges', () => {
    render(SignalCard, { props: { signal: mockSignal } });
    
    expect(screen.getByText('EMA Bullish')).toBeInTheDocument();
    expect(screen.getByText('RSI Oversold')).toBeInTheDocument();
  });

  it('displays signal reason', () => {
    render(SignalCard, { props: { signal: mockSignal } });
    
    expect(screen.getByText('Strong accumulation pattern')).toBeInTheDocument();
  });

  it('shows strength indicator', () => {
    render(SignalCard, { props: { signal: mockSignal } });
    
    // Strong signal shows ↑↑↑
    expect(screen.getByText('↑↑↑')).toBeInTheDocument();
  });

  it('renders sell signal with correct styling', () => {
    const sellSignal = {
      ...mockSignal,
      type: 'sell' as const,
      score: 45,
    };
    
    render(SignalCard, { props: { signal: sellSignal } });
    
    expect(screen.getByText('SELL')).toBeInTheDocument();
    const scoreElement = screen.getByText('45');
    expect(scoreElement).toHaveClass('text-red-500');
  });

  it('handles signal without optional fields', () => {
    const minimalSignal = {
      id: '2',
      symbol: 'TLKM',
      stockName: 'Telkom Indonesia',
      type: 'hold' as const,
      strength: 'weak' as const,
      score: 55,
      reason: 'Consolidating',
      timestamp: new Date(),
      priceAtSignal: 3500,
      indicators: [],
    };
    
    render(SignalCard, { props: { signal: minimalSignal } });
    
    expect(screen.getByText('TLKM')).toBeInTheDocument();
    expect(screen.getByText('HOLD')).toBeInTheDocument();
    expect(screen.getByText('3,500')).toBeInTheDocument();
  });
});
