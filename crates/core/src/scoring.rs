//! Scoring engine for combining technical, fundamental, sentiment, and ML scores

/// Weights for composite score calculation
#[derive(Debug, Clone)]
pub struct ScoreWeights {
    pub technical: f64,
    pub fundamental: f64,
    pub sentiment: f64,
    pub ml: f64,
}

impl Default for ScoreWeights {
    fn default() -> Self {
        Self {
            technical: 0.40,
            fundamental: 0.40,
            sentiment: 0.10,
            ml: 0.10,
        }
    }
}

/// Calculate composite score from components
pub fn calculate_composite_score(
    technical: f64,
    fundamental: f64,
    sentiment: f64,
    ml: f64,
    weights: &ScoreWeights,
) -> f64 {
    technical * weights.technical
        + fundamental * weights.fundamental
        + sentiment * weights.sentiment
        + ml * weights.ml
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_weights() {
        let weights = ScoreWeights::default();
        let total = weights.technical + weights.fundamental + weights.sentiment + weights.ml;
        assert!((total - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_composite_score() {
        let weights = ScoreWeights::default();
        let score = calculate_composite_score(80.0, 70.0, 60.0, 50.0, &weights);
        // 80*0.4 + 70*0.4 + 60*0.1 + 50*0.1 = 32 + 28 + 6 + 5 = 71
        assert!((score - 71.0).abs() < f64::EPSILON);
    }
}
