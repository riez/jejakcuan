//! Fundamental Score Engine
//!
//! Combines valuation metrics into a 0-100 fundamental score:
//! - Valuation (P/E, P/B, EV/EBITDA vs sector) - 35%
//! - DCF Margin of Safety - 25%
//! - Quality (ROE, ROA, Profit Margin) - 20%
//! - Financial Health (D/E, Current Ratio) - 20%

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Weights for fundamental score components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundamentalWeights {
    pub valuation: Decimal,
    pub dcf: Decimal,
    pub quality: Decimal,
    pub health: Decimal,
}

impl Default for FundamentalWeights {
    fn default() -> Self {
        Self {
            valuation: dec!(0.35),
            dcf: dec!(0.25),
            quality: dec!(0.20),
            health: dec!(0.20),
        }
    }
}

/// Input data for fundamental scoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FundamentalInput {
    /// P/E ratio
    pub pe_ratio: Option<Decimal>,
    /// Sector average P/E
    pub sector_pe: Option<Decimal>,
    /// P/B ratio
    pub pb_ratio: Option<Decimal>,
    /// Sector average P/B
    pub sector_pb: Option<Decimal>,
    /// EV/EBITDA
    pub ev_ebitda: Option<Decimal>,
    /// Sector average EV/EBITDA
    pub sector_ev_ebitda: Option<Decimal>,
    /// DCF margin of safety (%)
    pub dcf_margin: Option<Decimal>,
    /// ROE (%)
    pub roe: Option<Decimal>,
    /// ROA (%)
    pub roa: Option<Decimal>,
    /// Profit margin (%)
    pub profit_margin: Option<Decimal>,
    /// Debt-to-Equity ratio
    pub debt_to_equity: Option<Decimal>,
    /// Current ratio
    pub current_ratio: Option<Decimal>,
}

/// Fundamental score result with breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundamentalScoreBreakdown {
    /// Overall fundamental score (0-100)
    pub total_score: Decimal,
    /// Valuation sub-score
    pub valuation_score: Decimal,
    /// DCF sub-score
    pub dcf_score: Decimal,
    /// Quality sub-score
    pub quality_score: Decimal,
    /// Financial health sub-score
    pub health_score: Decimal,
    /// Signals/explanations
    pub signals: Vec<String>,
    /// Assessment summary
    pub assessment: FundamentalAssessment,
}

/// Assessment category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FundamentalAssessment {
    Strong,
    Moderate,
    Weak,
    Insufficient,
}

/// Fundamental Score Engine
pub struct FundamentalScoreEngine {
    weights: FundamentalWeights,
}

impl FundamentalScoreEngine {
    /// Create new engine with default weights
    #[must_use]
    pub fn new() -> Self {
        Self {
            weights: FundamentalWeights::default(),
        }
    }

    /// Create engine with custom weights
    #[must_use]
    pub fn with_weights(weights: FundamentalWeights) -> Self {
        Self { weights }
    }

    /// Calculate fundamental score from input data
    #[must_use]
    pub fn calculate(&self, input: &FundamentalInput) -> FundamentalScoreBreakdown {
        let mut signals = Vec::new();

        // Calculate sub-scores
        let valuation_score = self.calculate_valuation_score(input, &mut signals);
        let dcf_score = self.calculate_dcf_score(input, &mut signals);
        let quality_score = self.calculate_quality_score(input, &mut signals);
        let health_score = self.calculate_health_score(input, &mut signals);

        // Weighted total
        let total_score = (valuation_score * self.weights.valuation
            + dcf_score * self.weights.dcf
            + quality_score * self.weights.quality
            + health_score * self.weights.health)
            .round_dp(2);

        // Determine assessment
        let assessment = if total_score >= dec!(70) {
            FundamentalAssessment::Strong
        } else if total_score >= dec!(50) {
            FundamentalAssessment::Moderate
        } else if total_score >= dec!(30) {
            FundamentalAssessment::Weak
        } else {
            FundamentalAssessment::Insufficient
        };

        FundamentalScoreBreakdown {
            total_score,
            valuation_score: valuation_score.round_dp(2),
            dcf_score: dcf_score.round_dp(2),
            quality_score: quality_score.round_dp(2),
            health_score: health_score.round_dp(2),
            signals,
            assessment,
        }
    }

    /// Calculate valuation score from ratios vs sector
    fn calculate_valuation_score(
        &self,
        input: &FundamentalInput,
        signals: &mut Vec<String>,
    ) -> Decimal {
        let mut total_score = Decimal::ZERO;
        let mut count = 0;

        // P/E scoring (lower is better, relative to sector)
        if let (Some(pe), Some(sector_pe)) = (input.pe_ratio, input.sector_pe) {
            let pe_score = if pe <= Decimal::ZERO {
                dec!(0) // Negative earnings = 0
            } else if sector_pe <= Decimal::ZERO {
                dec!(50) // Can't compare, neutral
            } else {
                let ratio = pe / sector_pe;
                if ratio < dec!(0.5) {
                    signals.push(format!("P/E ({pe}) very low vs sector ({sector_pe})"));
                    dec!(100)
                } else if ratio < dec!(0.7) {
                    signals.push(format!("P/E ({pe}) low vs sector ({sector_pe})"));
                    dec!(85)
                } else if ratio < dec!(0.9) {
                    dec!(70)
                } else if ratio < dec!(1.1) {
                    dec!(60)
                } else if ratio < dec!(1.3) {
                    dec!(45)
                } else {
                    signals.push(format!("P/E ({pe}) high vs sector ({sector_pe})"));
                    dec!(30)
                }
            };
            total_score += pe_score;
            count += 1;
        }

        // P/B scoring
        if let (Some(pb), Some(sector_pb)) = (input.pb_ratio, input.sector_pb) {
            let pb_score = if pb <= Decimal::ZERO {
                dec!(0)
            } else if sector_pb <= Decimal::ZERO {
                dec!(50) // Can't compare, neutral
            } else {
                let ratio = pb / sector_pb;
                if pb < dec!(1) {
                    signals.push(format!("Trading below book value (P/B: {pb})"));
                    dec!(90)
                } else if ratio < dec!(0.7) {
                    dec!(85)
                } else if ratio < dec!(1.0) {
                    dec!(70)
                } else if ratio < dec!(1.3) {
                    dec!(55)
                } else {
                    dec!(35)
                }
            };
            total_score += pb_score;
            count += 1;
        }

        // EV/EBITDA scoring
        if let (Some(ev), Some(sector_ev)) = (input.ev_ebitda, input.sector_ev_ebitda) {
            let ev_score = if ev <= Decimal::ZERO {
                dec!(0)
            } else if sector_ev <= Decimal::ZERO {
                dec!(50) // Can't compare, neutral
            } else {
                let ratio = ev / sector_ev;
                if ev < dec!(6) {
                    signals.push(format!("EV/EBITDA ({ev}) attractive"));
                    dec!(90)
                } else if ratio < dec!(0.7) {
                    dec!(85)
                } else if ratio < dec!(1.0) {
                    dec!(70)
                } else if ratio < dec!(1.3) {
                    dec!(55)
                } else {
                    dec!(35)
                }
            };
            total_score += ev_score;
            count += 1;
        }

        if count > 0 {
            (total_score / Decimal::from(count)).round_dp(2)
        } else {
            dec!(50) // Neutral if no data
        }
    }

    /// Calculate DCF-based score
    fn calculate_dcf_score(&self, input: &FundamentalInput, signals: &mut Vec<String>) -> Decimal {
        match input.dcf_margin {
            Some(margin) => {
                if margin >= dec!(30) {
                    signals.push(format!("Strong margin of safety ({margin}%)"));
                    dec!(100)
                } else if margin >= dec!(20) {
                    signals.push(format!("Good margin of safety ({margin}%)"));
                    dec!(85)
                } else if margin >= dec!(10) {
                    dec!(70)
                } else if margin >= dec!(0) {
                    dec!(55)
                } else if margin >= dec!(-10) {
                    dec!(40)
                } else {
                    signals.push(format!("Overvalued by DCF ({margin}%)"));
                    dec!(25)
                }
            }
            None => dec!(50), // Neutral if no DCF data
        }
    }

    /// Calculate quality score (ROE, ROA, Profit Margin)
    fn calculate_quality_score(
        &self,
        input: &FundamentalInput,
        signals: &mut Vec<String>,
    ) -> Decimal {
        let mut total_score = Decimal::ZERO;
        let mut count = 0;

        // ROE scoring (higher is better)
        if let Some(roe) = input.roe {
            let roe_score = if roe >= dec!(25) {
                signals.push(format!("Excellent ROE ({roe}%)"));
                dec!(100)
            } else if roe >= dec!(15) {
                dec!(80)
            } else if roe >= dec!(10) {
                dec!(60)
            } else if roe >= dec!(5) {
                dec!(40)
            } else {
                signals.push(format!("Low ROE ({roe}%)"));
                dec!(20)
            };
            total_score += roe_score;
            count += 1;
        }

        // ROA scoring
        if let Some(roa) = input.roa {
            let roa_score = if roa >= dec!(15) {
                dec!(100)
            } else if roa >= dec!(10) {
                dec!(80)
            } else if roa >= dec!(5) {
                dec!(60)
            } else if roa >= dec!(2) {
                dec!(40)
            } else {
                dec!(20)
            };
            total_score += roa_score;
            count += 1;
        }

        // Profit margin scoring
        if let Some(pm) = input.profit_margin {
            let pm_score = if pm >= dec!(20) {
                signals.push(format!("High profit margin ({pm}%)"));
                dec!(100)
            } else if pm >= dec!(10) {
                dec!(75)
            } else if pm >= dec!(5) {
                dec!(55)
            } else if pm >= dec!(0) {
                dec!(35)
            } else {
                signals.push("Negative profit margin".to_string());
                dec!(10)
            };
            total_score += pm_score;
            count += 1;
        }

        if count > 0 {
            (total_score / Decimal::from(count)).round_dp(2)
        } else {
            dec!(50)
        }
    }

    /// Calculate financial health score
    fn calculate_health_score(
        &self,
        input: &FundamentalInput,
        signals: &mut Vec<String>,
    ) -> Decimal {
        let mut total_score = Decimal::ZERO;
        let mut count = 0;

        // Debt-to-Equity scoring (lower is generally better)
        if let Some(de) = input.debt_to_equity {
            let de_score = if de <= dec!(0.3) {
                signals.push("Very low leverage".to_string());
                dec!(100)
            } else if de <= dec!(0.5) {
                dec!(85)
            } else if de <= dec!(1.0) {
                dec!(70)
            } else if de <= dec!(1.5) {
                dec!(55)
            } else if de <= dec!(2.0) {
                dec!(40)
            } else {
                signals.push(format!("High leverage (D/E: {de})"));
                dec!(25)
            };
            total_score += de_score;
            count += 1;
        }

        // Current ratio scoring (higher is better, but too high may indicate inefficiency)
        if let Some(cr) = input.current_ratio {
            let cr_score = if cr >= dec!(1.5) && cr <= dec!(3.0) {
                dec!(90)
            } else if cr >= dec!(1.2) && cr < dec!(1.5) {
                dec!(75)
            } else if cr >= dec!(1.0) && cr < dec!(1.2) {
                dec!(60)
            } else if cr >= dec!(0.8) && cr < dec!(1.0) {
                signals.push(format!("Low liquidity (CR: {cr})"));
                dec!(40)
            } else if cr < dec!(0.8) {
                signals.push(format!("Liquidity concern (CR: {cr})"));
                dec!(20)
            } else {
                // CR > 3.0 - possibly too much idle cash
                dec!(70)
            };
            total_score += cr_score;
            count += 1;
        }

        if count > 0 {
            (total_score / Decimal::from(count)).round_dp(2)
        } else {
            dec!(50)
        }
    }
}

impl Default for FundamentalScoreEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_input() -> FundamentalInput {
        FundamentalInput {
            pe_ratio: Some(dec!(10)),
            sector_pe: Some(dec!(15)),
            pb_ratio: Some(dec!(0.8)),
            sector_pb: Some(dec!(2)),
            ev_ebitda: Some(dec!(5)),
            sector_ev_ebitda: Some(dec!(10)),
            dcf_margin: Some(dec!(25)),
            roe: Some(dec!(20)),
            roa: Some(dec!(12)),
            profit_margin: Some(dec!(15)),
            debt_to_equity: Some(dec!(0.4)),
            current_ratio: Some(dec!(1.8)),
        }
    }

    #[test]
    fn test_default_weights_sum_to_one() {
        let weights = FundamentalWeights::default();
        let sum = weights.valuation + weights.dcf + weights.quality + weights.health;
        assert_eq!(sum, dec!(1));
    }

    #[test]
    fn test_strong_fundamentals() {
        let engine = FundamentalScoreEngine::new();
        let input = test_input();
        let result = engine.calculate(&input);

        assert!(result.total_score >= dec!(70));
        assert_eq!(result.assessment, FundamentalAssessment::Strong);
        assert!(!result.signals.is_empty());
    }

    #[test]
    fn test_weak_fundamentals() {
        let engine = FundamentalScoreEngine::new();
        let input = FundamentalInput {
            pe_ratio: Some(dec!(30)),
            sector_pe: Some(dec!(15)),
            pb_ratio: Some(dec!(4)),
            sector_pb: Some(dec!(2)),
            ev_ebitda: Some(dec!(20)),
            sector_ev_ebitda: Some(dec!(10)),
            dcf_margin: Some(dec!(-20)),
            roe: Some(dec!(3)),
            roa: Some(dec!(1)),
            profit_margin: Some(dec!(2)),
            debt_to_equity: Some(dec!(2.5)),
            current_ratio: Some(dec!(0.7)),
        };
        let result = engine.calculate(&input);

        assert!(result.total_score < dec!(50));
        assert!(matches!(
            result.assessment,
            FundamentalAssessment::Weak | FundamentalAssessment::Insufficient
        ));
    }

    #[test]
    fn test_missing_data_neutral() {
        let engine = FundamentalScoreEngine::new();
        let input = FundamentalInput::default();
        let result = engine.calculate(&input);

        // With all missing data, should be neutral (50)
        assert_eq!(result.total_score, dec!(50));
    }

    #[test]
    fn test_custom_weights() {
        let engine = FundamentalScoreEngine::with_weights(FundamentalWeights {
            valuation: dec!(0.50),
            dcf: dec!(0.30),
            quality: dec!(0.10),
            health: dec!(0.10),
        });
        let input = test_input();
        let result = engine.calculate(&input);

        assert!(result.total_score > Decimal::ZERO);
    }

    #[test]
    fn test_score_bounds() {
        let engine = FundamentalScoreEngine::new();

        // Test extreme strong case
        let strong_input = FundamentalInput {
            pe_ratio: Some(dec!(5)),
            sector_pe: Some(dec!(20)),
            pb_ratio: Some(dec!(0.5)),
            sector_pb: Some(dec!(3)),
            ev_ebitda: Some(dec!(4)),
            sector_ev_ebitda: Some(dec!(15)),
            dcf_margin: Some(dec!(50)),
            roe: Some(dec!(30)),
            roa: Some(dec!(20)),
            profit_margin: Some(dec!(25)),
            debt_to_equity: Some(dec!(0.1)),
            current_ratio: Some(dec!(2.0)),
        };

        let result = engine.calculate(&strong_input);
        assert!(result.total_score <= dec!(100));
        assert!(result.total_score >= dec!(0));

        // Test extreme weak case
        let weak_input = FundamentalInput {
            pe_ratio: Some(dec!(50)),
            sector_pe: Some(dec!(15)),
            pb_ratio: Some(dec!(6)),
            sector_pb: Some(dec!(2)),
            ev_ebitda: Some(dec!(30)),
            sector_ev_ebitda: Some(dec!(10)),
            dcf_margin: Some(dec!(-50)),
            roe: Some(dec!(1)),
            roa: Some(dec!(0)),
            profit_margin: Some(dec!(-5)),
            debt_to_equity: Some(dec!(5)),
            current_ratio: Some(dec!(0.5)),
        };

        let result = engine.calculate(&weak_input);
        assert!(result.total_score <= dec!(100));
        assert!(result.total_score >= dec!(0));
    }

    #[test]
    fn test_valuation_signals() {
        let engine = FundamentalScoreEngine::new();

        // Test P/E signal
        let input = FundamentalInput {
            pe_ratio: Some(dec!(5)),
            sector_pe: Some(dec!(15)),
            ..Default::default()
        };
        let result = engine.calculate(&input);
        assert!(result.signals.iter().any(|s| s.contains("P/E")));

        // Test P/B below book value signal
        let input = FundamentalInput {
            pb_ratio: Some(dec!(0.8)),
            sector_pb: Some(dec!(2)),
            ..Default::default()
        };
        let result = engine.calculate(&input);
        assert!(result
            .signals
            .iter()
            .any(|s| s.contains("below book value")));
    }

    #[test]
    fn test_dcf_signals() {
        let engine = FundamentalScoreEngine::new();

        // Test strong margin of safety
        let input = FundamentalInput {
            dcf_margin: Some(dec!(35)),
            ..Default::default()
        };
        let result = engine.calculate(&input);
        assert!(result
            .signals
            .iter()
            .any(|s| s.contains("Strong margin of safety")));

        // Test overvalued
        let input = FundamentalInput {
            dcf_margin: Some(dec!(-15)),
            ..Default::default()
        };
        let result = engine.calculate(&input);
        assert!(result.signals.iter().any(|s| s.contains("Overvalued")));
    }

    #[test]
    fn test_quality_signals() {
        let engine = FundamentalScoreEngine::new();

        // Test excellent ROE
        let input = FundamentalInput {
            roe: Some(dec!(28)),
            ..Default::default()
        };
        let result = engine.calculate(&input);
        assert!(result.signals.iter().any(|s| s.contains("Excellent ROE")));

        // Test high profit margin
        let input = FundamentalInput {
            profit_margin: Some(dec!(22)),
            ..Default::default()
        };
        let result = engine.calculate(&input);
        assert!(result
            .signals
            .iter()
            .any(|s| s.contains("High profit margin")));
    }

    #[test]
    fn test_health_signals() {
        let engine = FundamentalScoreEngine::new();

        // Test very low leverage
        let input = FundamentalInput {
            debt_to_equity: Some(dec!(0.2)),
            ..Default::default()
        };
        let result = engine.calculate(&input);
        assert!(result
            .signals
            .iter()
            .any(|s| s.contains("Very low leverage")));

        // Test liquidity concern
        let input = FundamentalInput {
            current_ratio: Some(dec!(0.6)),
            ..Default::default()
        };
        let result = engine.calculate(&input);
        assert!(result
            .signals
            .iter()
            .any(|s| s.contains("Liquidity concern")));
    }

    #[test]
    fn test_negative_pe_zero_score() {
        let engine = FundamentalScoreEngine::new();
        let input = FundamentalInput {
            pe_ratio: Some(dec!(-5)),
            sector_pe: Some(dec!(15)),
            ..Default::default()
        };
        let result = engine.calculate(&input);
        // Valuation score should be 0 for negative P/E
        assert_eq!(result.valuation_score, dec!(0));
    }

    #[test]
    fn test_assessment_categories() {
        let engine = FundamentalScoreEngine::new();

        // Strong (>= 70)
        let strong_input = test_input();
        let result = engine.calculate(&strong_input);
        assert_eq!(result.assessment, FundamentalAssessment::Strong);

        // Moderate (50-69)
        let moderate_input = FundamentalInput {
            pe_ratio: Some(dec!(15)),
            sector_pe: Some(dec!(15)),
            pb_ratio: Some(dec!(2)),
            sector_pb: Some(dec!(2)),
            ev_ebitda: Some(dec!(10)),
            sector_ev_ebitda: Some(dec!(10)),
            dcf_margin: Some(dec!(5)),
            roe: Some(dec!(12)),
            roa: Some(dec!(7)),
            profit_margin: Some(dec!(8)),
            debt_to_equity: Some(dec!(0.8)),
            current_ratio: Some(dec!(1.3)),
        };
        let result = engine.calculate(&moderate_input);
        assert_eq!(result.assessment, FundamentalAssessment::Moderate);

        // Weak (30-49)
        let weak_input = FundamentalInput {
            pe_ratio: Some(dec!(25)),
            sector_pe: Some(dec!(15)),
            pb_ratio: Some(dec!(3)),
            sector_pb: Some(dec!(2)),
            ev_ebitda: Some(dec!(15)),
            sector_ev_ebitda: Some(dec!(10)),
            dcf_margin: Some(dec!(-5)),
            roe: Some(dec!(6)),
            roa: Some(dec!(3)),
            profit_margin: Some(dec!(4)),
            debt_to_equity: Some(dec!(1.8)),
            current_ratio: Some(dec!(0.9)),
        };
        let result = engine.calculate(&weak_input);
        assert_eq!(result.assessment, FundamentalAssessment::Weak);
    }
}
