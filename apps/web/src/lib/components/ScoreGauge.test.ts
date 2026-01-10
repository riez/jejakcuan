import { render, screen } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import ScoreGauge from './ScoreGauge.svelte';

describe('ScoreGauge', () => {
  it('renders the score value', () => {
    render(ScoreGauge, { props: { score: 75, label: 'Test Score' } });
    
    expect(screen.getByText('75')).toBeInTheDocument();
    expect(screen.getByText('Test Score')).toBeInTheDocument();
  });

  it('shows green color for high scores (>=70)', () => {
    const { container } = render(ScoreGauge, { props: { score: 80, label: 'Score' } });
    
    const circle = container.querySelector('circle.stroke-green-500');
    expect(circle).toBeInTheDocument();
  });

  it('shows yellow color for medium scores (50-69)', () => {
    const { container } = render(ScoreGauge, { props: { score: 55, label: 'Score' } });
    
    const circle = container.querySelector('circle.stroke-yellow-500');
    expect(circle).toBeInTheDocument();
  });

  it('shows red color for low scores (<50)', () => {
    const { container } = render(ScoreGauge, { props: { score: 30, label: 'Score' } });
    
    const circle = container.querySelector('circle.stroke-red-500');
    expect(circle).toBeInTheDocument();
  });

  it('uses default values when not provided', () => {
    render(ScoreGauge, { props: {} });
    
    // Default score is 50
    expect(screen.getByText('50')).toBeInTheDocument();
    // Default label is 'Score'
    expect(screen.getByText('Score')).toBeInTheDocument();
  });

  it('applies the correct size class', () => {
    const { container } = render(ScoreGauge, { props: { score: 50, size: 'lg' } });
    
    const wrapper = container.querySelector('.w-32.h-32');
    expect(wrapper).toBeInTheDocument();
  });
});
