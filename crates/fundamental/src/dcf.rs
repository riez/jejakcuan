//! Discounted Cash Flow (DCF) Valuation Model
//!
//! Calculates intrinsic value using:
//! - Free Cash Flow projection
//! - WACC (Weighted Average Cost of Capital)
//! - Terminal Value calculation
//! - Margin of Safety analysis

use crate::error::FundamentalError;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// DCF Input parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DcfInput {
    /// Current Free Cash Flow (latest annual)
    pub current_fcf: Decimal,
    /// Number of shares outstanding
    pub shares_outstanding: i64,
    /// Current stock price
    pub current_price: Decimal,
    /// Historical FCF growth rates (for estimating future growth)
    pub historical_growth_rates: Vec<Decimal>,
    /// Cost of equity (risk-free rate + beta * market risk premium)
    pub cost_of_equity: Option<Decimal>,
    /// Cost of debt (interest expense / total debt)
    pub cost_of_debt: Option<Decimal>,
    /// Tax rate
    pub tax_rate: Option<Decimal>,
    /// Debt ratio (total debt / (debt + equity))
    pub debt_ratio: Option<Decimal>,
    /// Terminal growth rate (usually GDP growth rate)
    pub terminal_growth_rate: Option<Decimal>,
    /// Projection years (default: 5)
    pub projection_years: Option<usize>,
}

/// DCF calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DcfResult {
    /// Intrinsic value per share
    pub intrinsic_value: Decimal,
    /// Current market price
    pub current_price: Decimal,
    /// Margin of safety percentage
    pub margin_of_safety: Decimal,
    /// Whether stock appears undervalued
    pub is_undervalued: bool,
    /// Projected FCF for each year
    pub projected_fcf: Vec<Decimal>,
    /// Present value of projected FCF
    pub pv_fcf: Vec<Decimal>,
    /// Terminal value
    pub terminal_value: Decimal,
    /// Present value of terminal value
    pub pv_terminal_value: Decimal,
    /// Total enterprise value
    pub enterprise_value: Decimal,
    /// WACC used in calculation
    pub wacc: Decimal,
    /// Growth rate used
    pub growth_rate: Decimal,
    /// Assumptions used
    pub assumptions: DcfAssumptions,
}

/// DCF assumptions for transparency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DcfAssumptions {
    pub growth_rate: Decimal,
    pub terminal_growth_rate: Decimal,
    pub wacc: Decimal,
    pub projection_years: usize,
    pub risk_free_rate: Decimal,
    pub market_risk_premium: Decimal,
    pub beta: Decimal,
}

/// DCF sensitivity analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DcfSensitivity {
    pub base_value: Decimal,
    /// Values at different growth rates
    pub growth_sensitivity: Vec<(Decimal, Decimal)>,
    /// Values at different WACC rates
    pub wacc_sensitivity: Vec<(Decimal, Decimal)>,
}

/// Default assumptions for Indonesian market
pub struct IndonesianMarketDefaults;

impl IndonesianMarketDefaults {
    /// Risk-free rate (10-year Indonesian government bond yield)
    pub const RISK_FREE_RATE: Decimal = dec!(6.5); // ~6.5%

    /// Market risk premium for Indonesian market
    pub const MARKET_RISK_PREMIUM: Decimal = dec!(7.0); // ~7%

    /// Average corporate tax rate
    pub const TAX_RATE: Decimal = dec!(22); // 22%

    /// Long-term GDP growth rate (terminal growth)
    pub const TERMINAL_GROWTH: Decimal = dec!(5.0); // ~5%

    /// Default beta for Indonesian stocks
    pub const DEFAULT_BETA: Decimal = dec!(1.0);
}

/// Calculate WACC (Weighted Average Cost of Capital)
/// WACC = (E/V) * Re + (D/V) * Rd * (1 - Tc)
pub fn calculate_wacc(
    cost_of_equity: Decimal,
    cost_of_debt: Decimal,
    tax_rate: Decimal,
    debt_ratio: Decimal,
) -> Decimal {
    let equity_ratio = dec!(1) - debt_ratio;
    let after_tax_cost_of_debt = cost_of_debt * (dec!(1) - tax_rate / dec!(100));

    ((equity_ratio * cost_of_equity) + (debt_ratio * after_tax_cost_of_debt)).round_dp(2)
}

/// Calculate Cost of Equity using CAPM
/// Re = Rf + β * (Rm - Rf)
pub fn calculate_cost_of_equity(
    risk_free_rate: Decimal,
    beta: Decimal,
    market_risk_premium: Decimal,
) -> Decimal {
    (risk_free_rate + beta * market_risk_premium).round_dp(2)
}

/// Estimate growth rate from historical data
pub fn estimate_growth_rate(historical_rates: &[Decimal]) -> Decimal {
    if historical_rates.is_empty() {
        return dec!(5); // Default 5% growth
    }

    // Use arithmetic mean for growth estimation
    let sum: Decimal = historical_rates.iter().sum();
    let avg = sum / Decimal::from(historical_rates.len() as i64);

    // Cap growth rate between -10% and 30%
    avg.max(dec!(-10)).min(dec!(30)).round_dp(2)
}

/// Calculate DCF valuation
pub fn calculate_dcf(input: &DcfInput) -> Result<DcfResult, FundamentalError> {
    if input.current_fcf <= Decimal::ZERO {
        return Err(FundamentalError::InvalidValue(
            "FCF must be positive for DCF valuation".to_string(),
        ));
    }

    if input.shares_outstanding <= 0 {
        return Err(FundamentalError::InvalidValue(
            "Shares outstanding must be positive".to_string(),
        ));
    }

    let projection_years = input.projection_years.unwrap_or(5);

    // Estimate growth rate
    let growth_rate = estimate_growth_rate(&input.historical_growth_rates);
    let growth_decimal = growth_rate / dec!(100);

    // Terminal growth rate
    let terminal_growth = input
        .terminal_growth_rate
        .unwrap_or(IndonesianMarketDefaults::TERMINAL_GROWTH)
        / dec!(100);

    // Calculate cost of equity
    let risk_free = IndonesianMarketDefaults::RISK_FREE_RATE;
    let market_premium = IndonesianMarketDefaults::MARKET_RISK_PREMIUM;
    let beta = IndonesianMarketDefaults::DEFAULT_BETA;

    let cost_of_equity = input
        .cost_of_equity
        .unwrap_or_else(|| calculate_cost_of_equity(risk_free, beta, market_premium));

    // Calculate WACC
    let cost_of_debt = input.cost_of_debt.unwrap_or(dec!(8)); // Default 8%
    let tax_rate = input.tax_rate.unwrap_or(IndonesianMarketDefaults::TAX_RATE);
    let debt_ratio = input.debt_ratio.unwrap_or(dec!(0.3)); // Default 30% debt

    let wacc = calculate_wacc(cost_of_equity, cost_of_debt, tax_rate, debt_ratio);
    let wacc_decimal = wacc / dec!(100);

    // Project FCF for each year
    let mut projected_fcf = Vec::with_capacity(projection_years);
    let mut current_fcf = input.current_fcf;

    for _ in 0..projection_years {
        current_fcf *= dec!(1) + growth_decimal;
        projected_fcf.push(current_fcf.round_dp(0));
    }

    // Calculate present value of projected FCF
    let mut pv_fcf = Vec::with_capacity(projection_years);
    let mut total_pv_fcf = Decimal::ZERO;

    for (i, fcf) in projected_fcf.iter().enumerate() {
        let discount_factor = power_decimal(dec!(1) + wacc_decimal, i as i32 + 1);
        let pv = *fcf / discount_factor;
        pv_fcf.push(pv.round_dp(0));
        total_pv_fcf += pv;
    }

    // Calculate terminal value using Gordon Growth Model
    // TV = FCF(n+1) / (WACC - g)
    let terminal_fcf =
        projected_fcf.last().copied().unwrap_or(input.current_fcf) * (dec!(1) + terminal_growth);
    let terminal_value = if wacc_decimal > terminal_growth {
        terminal_fcf / (wacc_decimal - terminal_growth)
    } else {
        // Fallback: use multiple of final year FCF
        terminal_fcf * dec!(15)
    };

    // Present value of terminal value
    let terminal_discount = power_decimal(dec!(1) + wacc_decimal, projection_years as i32);
    let pv_terminal_value = terminal_value / terminal_discount;

    // Enterprise value = PV of FCF + PV of Terminal Value
    let enterprise_value = total_pv_fcf + pv_terminal_value;

    // Intrinsic value per share
    let intrinsic_value = (enterprise_value / Decimal::from(input.shares_outstanding)).round_dp(0);

    // Margin of safety
    let margin_of_safety = if input.current_price > Decimal::ZERO {
        ((intrinsic_value - input.current_price) / intrinsic_value * dec!(100)).round_dp(2)
    } else {
        Decimal::ZERO
    };

    let is_undervalued = intrinsic_value > input.current_price;

    Ok(DcfResult {
        intrinsic_value,
        current_price: input.current_price,
        margin_of_safety,
        is_undervalued,
        projected_fcf,
        pv_fcf,
        terminal_value: terminal_value.round_dp(0),
        pv_terminal_value: pv_terminal_value.round_dp(0),
        enterprise_value: enterprise_value.round_dp(0),
        wacc,
        growth_rate,
        assumptions: DcfAssumptions {
            growth_rate,
            terminal_growth_rate: terminal_growth * dec!(100),
            wacc,
            projection_years,
            risk_free_rate: risk_free,
            market_risk_premium: market_premium,
            beta,
        },
    })
}

/// Calculate DCF sensitivity analysis
pub fn calculate_sensitivity(input: &DcfInput, base_result: &DcfResult) -> DcfSensitivity {
    let mut growth_sensitivity = Vec::new();
    let mut wacc_sensitivity = Vec::new();

    // Growth rate sensitivity (-5% to +5% from base)
    for delta in [-5, -2, 0, 2, 5] {
        let mut modified_input = input.clone();
        let new_growth: Vec<Decimal> = input
            .historical_growth_rates
            .iter()
            .map(|g| *g + Decimal::from(delta))
            .collect();
        modified_input.historical_growth_rates = new_growth;

        if let Ok(result) = calculate_dcf(&modified_input) {
            growth_sensitivity.push((
                base_result.growth_rate + Decimal::from(delta),
                result.intrinsic_value,
            ));
        }
    }

    // WACC sensitivity is harder to modify directly, so we'll show the base case
    wacc_sensitivity.push((base_result.wacc, base_result.intrinsic_value));

    DcfSensitivity {
        base_value: base_result.intrinsic_value,
        growth_sensitivity,
        wacc_sensitivity,
    }
}

/// Simple power function for Decimal
fn power_decimal(base: Decimal, exp: i32) -> Decimal {
    if exp == 0 {
        return dec!(1);
    }

    let mut result = dec!(1);
    for _ in 0..exp {
        result *= base;
    }
    result
}

/// Calculate margin of safety score (0-100)
/// Higher margin of safety = higher score
pub fn margin_of_safety_score(margin: Decimal) -> Decimal {
    // Margin of safety scoring:
    // >= 30% = 100 points (very undervalued)
    // 20-30% = 80-100 points
    // 10-20% = 60-80 points
    // 0-10% = 50-60 points
    // < 0% (overvalued) = 0-50 points

    if margin >= dec!(30) {
        dec!(100)
    } else if margin >= dec!(20) {
        dec!(80) + ((margin - dec!(20)) / dec!(10) * dec!(20))
    } else if margin >= dec!(10) {
        dec!(60) + ((margin - dec!(10)) / dec!(10) * dec!(20))
    } else if margin >= dec!(0) {
        dec!(50) + (margin / dec!(10) * dec!(10))
    } else {
        // Negative margin (overvalued)
        (dec!(50) + margin).max(Decimal::ZERO)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wacc_calculation() {
        let wacc = calculate_wacc(
            dec!(13.5), // cost of equity
            dec!(8),    // cost of debt
            dec!(22),   // tax rate
            dec!(0.3),  // 30% debt
        );

        // WACC should be between cost of debt and cost of equity
        assert!(wacc > dec!(8) && wacc < dec!(14));
    }

    #[test]
    fn test_cost_of_equity() {
        let coe = calculate_cost_of_equity(
            dec!(6.5), // risk-free rate
            dec!(1.2), // beta
            dec!(7),   // market risk premium
        );

        // Re = 6.5 + 1.2 * 7 = 14.9
        assert_eq!(coe, dec!(14.9));
    }

    #[test]
    fn test_growth_estimation() {
        let rates = vec![dec!(10), dec!(15), dec!(12), dec!(8), dec!(10)];
        let growth = estimate_growth_rate(&rates);

        // Average should be 11%
        assert_eq!(growth, dec!(11));
    }

    #[test]
    fn test_dcf_calculation() {
        let input = DcfInput {
            current_fcf: dec!(1_000_000_000), // 1 billion
            shares_outstanding: 10_000_000,
            current_price: dec!(8000),
            historical_growth_rates: vec![dec!(10), dec!(12), dec!(8), dec!(15), dec!(10)],
            cost_of_equity: None,
            cost_of_debt: None,
            tax_rate: None,
            debt_ratio: None,
            terminal_growth_rate: None,
            projection_years: Some(5),
        };

        let result = calculate_dcf(&input).unwrap();

        assert!(result.intrinsic_value > Decimal::ZERO);
        assert_eq!(result.projected_fcf.len(), 5);
        assert_eq!(result.pv_fcf.len(), 5);
    }

    #[test]
    fn test_margin_of_safety_score() {
        assert_eq!(margin_of_safety_score(dec!(30)), dec!(100));
        assert!(margin_of_safety_score(dec!(15)) > dec!(60));
        assert!(margin_of_safety_score(dec!(-10)) < dec!(50));
    }

    #[test]
    fn test_power_decimal() {
        assert_eq!(power_decimal(dec!(2), 3), dec!(8));
        assert_eq!(power_decimal(dec!(1.1), 2), dec!(1.21));
    }

    #[test]
    fn test_dcf_negative_fcf_error() {
        let input = DcfInput {
            current_fcf: dec!(-1_000_000),
            shares_outstanding: 10_000_000,
            current_price: dec!(8000),
            historical_growth_rates: vec![],
            cost_of_equity: None,
            cost_of_debt: None,
            tax_rate: None,
            debt_ratio: None,
            terminal_growth_rate: None,
            projection_years: None,
        };

        let result = calculate_dcf(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_dcf_zero_shares_error() {
        let input = DcfInput {
            current_fcf: dec!(1_000_000_000),
            shares_outstanding: 0,
            current_price: dec!(8000),
            historical_growth_rates: vec![],
            cost_of_equity: None,
            cost_of_debt: None,
            tax_rate: None,
            debt_ratio: None,
            terminal_growth_rate: None,
            projection_years: None,
        };

        let result = calculate_dcf(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_sensitivity_analysis() {
        let input = DcfInput {
            current_fcf: dec!(1_000_000_000),
            shares_outstanding: 10_000_000,
            current_price: dec!(8000),
            historical_growth_rates: vec![dec!(10), dec!(12), dec!(8)],
            cost_of_equity: None,
            cost_of_debt: None,
            tax_rate: None,
            debt_ratio: None,
            terminal_growth_rate: None,
            projection_years: Some(5),
        };

        let base_result = calculate_dcf(&input).unwrap();
        let sensitivity = calculate_sensitivity(&input, &base_result);

        assert!(!sensitivity.growth_sensitivity.is_empty());
        assert_eq!(sensitivity.base_value, base_result.intrinsic_value);
    }
}
