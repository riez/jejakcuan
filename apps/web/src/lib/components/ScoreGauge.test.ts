import { render, screen } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import ScoreGauge from './ScoreGauge.svelte';

describe('ScoreGauge', () => {
  it('renders the score value', () => {
    render(ScoreGauge, { props: { score: 75, label: 'Test Score' } });
    
    expect(screen.getByText('75')).toBeInTheDocument();
    expect(screen.getByText('Test Score')).toBeInTheDocument();
  });

  it('shows success color for high scores (>=70)', () => {
    const { container } = render(ScoreGauge, { props: { score: 80, label: 'Score' } });
    
    // ProgressRadial applies stroke class to SVG circle
    const scoreText = container.querySelector('.text-success-500');
    expect(scoreText).toBeInTheDocument();
  });

  it('shows warning color for medium scores (50-69)', () => {
    const { container } = render(ScoreGauge, { props: { score: 55, label: 'Score' } });
    
    const scoreText = container.querySelector('.text-warning-500');
    expect(scoreText).toBeInTheDocument();
  });

  it('shows error color for low scores (<50)', () => {
    const { container } = render(ScoreGauge, { props: { score: 30, label: 'Score' } });
    
    const scoreText = container.querySelector('.text-error-500');
    expect(scoreText).toBeInTheDocument();
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
    
    // Size class is applied via the width prop to ProgressRadial
    const progressRadial = container.querySelector('.w-28');
    expect(progressRadial).toBeInTheDocument();
  });
});
