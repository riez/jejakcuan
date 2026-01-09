//! Fundamental valuation metrics

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Financial data for a company
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialData {
    pub symbol: String,
    pub market_cap: Decimal,
    pub enterprise_value: Option<Decimal>,
    pub revenue: Decimal,
    pub net_income: Decimal,
    pub ebitda: Option<Decimal>,
    pub total_equity: Decimal,
    pub total_assets: Decimal,
    pub total_debt: Decimal,
    pub cash: Decimal,
    pub shares_outstanding: i64,
    pub eps: Decimal,
    pub book_value_per_share: Decimal,
    pub current_price: Decimal,
}

/// Calculated valuation ratios
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValuationRatios {
    pub pe_ratio: Option<Decimal>,
    pub forward_pe: Option<Decimal>,
    pub pb_ratio: Option<Decimal>,
    pub ps_ratio: Option<Decimal>,
    pub ev_ebitda: Option<Decimal>,
    pub ev_revenue: Option<Decimal>,
    pub roe: Option<Decimal>,
    pub roa: Option<Decimal>,
    pub profit_margin: Option<Decimal>,
    pub debt_to_equity: Option<Decimal>,
}

/// Valuation assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuationAssessment {
    pub pe_assessment: String,
    pub pb_assessment: String,
    pub ev_ebitda_assessment: String,
    pub overall_assessment: String,
    pub signals: Vec<String>,
}

/// Calculate P/E ratio
/// P/E = Price / Earnings per Share
pub fn calculate_pe_ratio(price: Decimal, eps: Decimal) -> Option<Decimal> {
    if eps <= Decimal::ZERO {
        return None; // Negative or zero earnings
    }
    Some((price / eps).round_dp(2))
}

/// Calculate Price-to-Book ratio
/// P/B = Price / Book Value per Share
pub fn calculate_pb_ratio(price: Decimal, book_value_per_share: Decimal) -> Option<Decimal> {
    if book_value_per_share <= Decimal::ZERO {
        return None;
    }
    Some((price / book_value_per_share).round_dp(2))
}

/// Calculate Price-to-Sales ratio
/// P/S = Market Cap / Revenue
pub fn calculate_ps_ratio(market_cap: Decimal, revenue: Decimal) -> Option<Decimal> {
    if revenue <= Decimal::ZERO {
        return None;
    }
    Some((market_cap / revenue).round_dp(2))
}

/// Calculate Enterprise Value
/// EV = Market Cap + Total Debt - Cash
pub fn calculate_enterprise_value(
    market_cap: Decimal,
    total_debt: Decimal,
    cash: Decimal,
) -> Decimal {
    market_cap + total_debt - cash
}

/// Calculate EV/EBITDA ratio
pub fn calculate_ev_ebitda(enterprise_value: Decimal, ebitda: Decimal) -> Option<Decimal> {
    if ebitda <= Decimal::ZERO {
        return None;
    }
    Some((enterprise_value / ebitda).round_dp(2))
}

/// Calculate EV/Revenue ratio
pub fn calculate_ev_revenue(enterprise_value: Decimal, revenue: Decimal) -> Option<Decimal> {
    if revenue <= Decimal::ZERO {
        return None;
    }
    Some((enterprise_value / revenue).round_dp(2))
}

/// Calculate Return on Equity
/// ROE = Net Income / Total Equity
pub fn calculate_roe(net_income: Decimal, total_equity: Decimal) -> Option<Decimal> {
    if total_equity <= Decimal::ZERO {
        return None;
    }
    Some(((net_income / total_equity) * dec!(100)).round_dp(2))
}

/// Calculate Return on Assets
/// ROA = Net Income / Total Assets
pub fn calculate_roa(net_income: Decimal, total_assets: Decimal) -> Option<Decimal> {
    if total_assets <= Decimal::ZERO {
        return None;
    }
    Some(((net_income / total_assets) * dec!(100)).round_dp(2))
}

/// Calculate Profit Margin
/// Profit Margin = Net Income / Revenue
pub fn calculate_profit_margin(net_income: Decimal, revenue: Decimal) -> Option<Decimal> {
    if revenue <= Decimal::ZERO {
        return None;
    }
    Some(((net_income / revenue) * dec!(100)).round_dp(2))
}

/// Calculate Debt-to-Equity ratio
pub fn calculate_debt_to_equity(total_debt: Decimal, total_equity: Decimal) -> Option<Decimal> {
    if total_equity <= Decimal::ZERO {
        return None;
    }
    Some((total_debt / total_equity).round_dp(2))
}

/// Calculate all valuation ratios from financial data
pub fn calculate_all_ratios(data: &FinancialData) -> ValuationRatios {
    let ev = data
        .enterprise_value
        .unwrap_or_else(|| calculate_enterprise_value(data.market_cap, data.total_debt, data.cash));

    ValuationRatios {
        pe_ratio: calculate_pe_ratio(data.current_price, data.eps),
        forward_pe: None, // Requires earnings estimates
        pb_ratio: calculate_pb_ratio(data.current_price, data.book_value_per_share),
        ps_ratio: calculate_ps_ratio(data.market_cap, data.revenue),
        ev_ebitda: data
            .ebitda
            .and_then(|ebitda| calculate_ev_ebitda(ev, ebitda)),
        ev_revenue: calculate_ev_revenue(ev, data.revenue),
        roe: calculate_roe(data.net_income, data.total_equity),
        roa: calculate_roa(data.net_income, data.total_assets),
        profit_margin: calculate_profit_margin(data.net_income, data.revenue),
        debt_to_equity: calculate_debt_to_equity(data.total_debt, data.total_equity),
    }
}

/// Assess valuation based on ratios and sector averages
pub fn assess_valuation(
    ratios: &ValuationRatios,
    sector_avg_pe: Option<Decimal>,
    sector_avg_pb: Option<Decimal>,
    sector_avg_ev_ebitda: Option<Decimal>,
) -> ValuationAssessment {
    let mut signals = Vec::new();

    // P/E Assessment
    let pe_assessment = match (ratios.pe_ratio, sector_avg_pe) {
        (Some(pe), Some(avg)) if pe < avg * dec!(0.7) => {
            signals.push(format!(
                "P/E ({}) significantly below sector avg ({})",
                pe, avg
            ));
            "undervalued".to_string()
        }
        (Some(pe), Some(avg)) if pe < avg => {
            signals.push(format!("P/E ({}) below sector avg ({})", pe, avg));
            "fairly_valued".to_string()
        }
        (Some(pe), Some(avg)) if pe > avg * dec!(1.3) => {
            signals.push(format!(
                "P/E ({}) significantly above sector avg ({})",
                pe, avg
            ));
            "overvalued".to_string()
        }
        (Some(pe), Some(avg)) => {
            signals.push(format!("P/E ({}) near sector avg ({})", pe, avg));
            "fairly_valued".to_string()
        }
        (Some(pe), None) => {
            if pe < dec!(10) {
                "potentially_undervalued".to_string()
            } else if pe > dec!(30) {
                "potentially_overvalued".to_string()
            } else {
                "unknown".to_string()
            }
        }
        (None, _) => "negative_earnings".to_string(),
    };

    // P/B Assessment
    let pb_assessment = match (ratios.pb_ratio, sector_avg_pb) {
        (Some(pb), Some(avg)) if pb < avg * dec!(0.7) => {
            signals.push(format!("P/B ({}) below sector avg ({})", pb, avg));
            "undervalued".to_string()
        }
        (Some(pb), Some(avg)) if pb > avg * dec!(1.3) => {
            signals.push(format!("P/B ({}) above sector avg ({})", pb, avg));
            "overvalued".to_string()
        }
        (Some(pb), _) if pb < dec!(1) => {
            signals.push("Trading below book value".to_string());
            "undervalued".to_string()
        }
        (Some(_), _) => "fairly_valued".to_string(),
        (None, _) => "unknown".to_string(),
    };

    // EV/EBITDA Assessment
    let ev_ebitda_assessment = match (ratios.ev_ebitda, sector_avg_ev_ebitda) {
        (Some(ev), Some(avg)) if ev < avg * dec!(0.7) => {
            signals.push(format!("EV/EBITDA ({}) below sector avg ({})", ev, avg));
            "undervalued".to_string()
        }
        (Some(ev), Some(avg)) if ev > avg * dec!(1.3) => {
            signals.push(format!("EV/EBITDA ({}) above sector avg ({})", ev, avg));
            "overvalued".to_string()
        }
        (Some(ev), _) if ev < dec!(8) => "attractive".to_string(),
        (Some(ev), _) if ev > dec!(15) => "expensive".to_string(),
        (Some(_), _) => "fairly_valued".to_string(),
        (None, _) => "unknown".to_string(),
    };

    // Overall assessment
    let assessments = [&pe_assessment, &pb_assessment, &ev_ebitda_assessment];
    let undervalued_count = assessments
        .iter()
        .filter(|a| a.contains("undervalued"))
        .count();
    let overvalued_count = assessments
        .iter()
        .filter(|a| a.contains("overvalued"))
        .count();

    let overall_assessment = if undervalued_count >= 2 {
        "undervalued"
    } else if overvalued_count >= 2 {
        "overvalued"
    } else {
        "fairly_valued"
    }
    .to_string();

    // Add ROE/ROA signals
    if let Some(roe) = ratios.roe {
        if roe > dec!(20) {
            signals.push(format!("Strong ROE: {}%", roe));
        } else if roe < dec!(5) {
            signals.push(format!("Weak ROE: {}%", roe));
        }
    }

    if let Some(de) = ratios.debt_to_equity {
        if de > dec!(2) {
            signals.push(format!("High leverage (D/E: {})", de));
        }
    }

    ValuationAssessment {
        pe_assessment,
        pb_assessment,
        ev_ebitda_assessment,
        overall_assessment,
        signals,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pe_ratio() {
        assert_eq!(calculate_pe_ratio(dec!(100), dec!(10)), Some(dec!(10)));
        assert_eq!(calculate_pe_ratio(dec!(100), dec!(-5)), None);
        assert_eq!(calculate_pe_ratio(dec!(100), dec!(0)), None);
    }

    #[test]
    fn test_pb_ratio() {
        assert_eq!(calculate_pb_ratio(dec!(100), dec!(50)), Some(dec!(2)));
        assert_eq!(calculate_pb_ratio(dec!(100), dec!(0)), None);
    }

    #[test]
    fn test_enterprise_value() {
        let ev = calculate_enterprise_value(dec!(1000), dec!(200), dec!(50));
        assert_eq!(ev, dec!(1150));
    }

    #[test]
    fn test_ev_ebitda() {
        assert_eq!(calculate_ev_ebitda(dec!(1000), dec!(100)), Some(dec!(10)));
        assert_eq!(calculate_ev_ebitda(dec!(1000), dec!(0)), None);
    }

    #[test]
    fn test_roe() {
        assert_eq!(calculate_roe(dec!(100), dec!(500)), Some(dec!(20)));
    }

    #[test]
    fn test_valuation_assessment() {
        let ratios = ValuationRatios {
            pe_ratio: Some(dec!(8)),
            forward_pe: None,
            pb_ratio: Some(dec!(0.8)),
            ps_ratio: Some(dec!(1)),
            ev_ebitda: Some(dec!(6)),
            ev_revenue: Some(dec!(1)),
            roe: Some(dec!(25)),
            roa: Some(dec!(10)),
            profit_margin: Some(dec!(15)),
            debt_to_equity: Some(dec!(0.5)),
        };

        let assessment = assess_valuation(
            &ratios,
            Some(dec!(15)), // sector avg P/E
            Some(dec!(2)),  // sector avg P/B
            Some(dec!(10)), // sector avg EV/EBITDA
        );

        assert_eq!(assessment.overall_assessment, "undervalued");
        assert!(!assessment.signals.is_empty());
    }
}
