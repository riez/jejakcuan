//! Shareholding analysis functions
//!
//! Analysis utilities for shareholding data to detect patterns
//! useful for investment decisions.

use super::models::{ChangeDirection, OwnershipChange, ShareholderType, ShareholdingSnapshot};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Ownership concentration metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcentrationMetrics {
    /// Herfindahl-Hirschman Index (HHI) for ownership concentration
    /// Higher values indicate more concentrated ownership
    pub hhi: Decimal,
    /// Top shareholder percentage
    pub top_1_percentage: Decimal,
    /// Top 3 shareholders combined percentage
    pub top_3_percentage: Decimal,
    /// Top 5 shareholders combined percentage
    pub top_5_percentage: Decimal,
    /// Top 10 shareholders combined percentage
    pub top_10_percentage: Decimal,
    /// Free float percentage
    pub free_float: Decimal,
}

impl ConcentrationMetrics {
    /// Calculate concentration metrics from shareholding snapshot
    pub fn from_snapshot(snapshot: &ShareholdingSnapshot) -> Self {
        let mut percentages: Vec<Decimal> =
            snapshot.shareholders.iter().map(|s| s.percentage).collect();
        percentages.sort_by(|a, b| b.cmp(a)); // Sort descending

        let hhi = Self::calculate_hhi(&percentages);

        let top_1_percentage = percentages.first().copied().unwrap_or(Decimal::ZERO);
        let top_3_percentage: Decimal = percentages.iter().take(3).sum();
        let top_5_percentage: Decimal = percentages.iter().take(5).sum();
        let top_10_percentage: Decimal = percentages.iter().take(10).sum();

        Self {
            hhi,
            top_1_percentage,
            top_3_percentage,
            top_5_percentage,
            top_10_percentage,
            free_float: snapshot.free_float,
        }
    }

    /// Calculate Herfindahl-Hirschman Index
    ///
    /// HHI ranges from 0 to 10,000 (using percentages squared)
    /// - < 1,500: Unconcentrated
    /// - 1,500-2,500: Moderately concentrated
    /// - > 2,500: Highly concentrated
    fn calculate_hhi(percentages: &[Decimal]) -> Decimal {
        percentages.iter().map(|p| p * p).sum()
    }

    /// Check if ownership is highly concentrated
    pub fn is_highly_concentrated(&self) -> bool {
        self.hhi > dec!(2500)
    }

    /// Check if ownership is moderately concentrated
    pub fn is_moderately_concentrated(&self) -> bool {
        self.hhi > dec!(1500) && self.hhi <= dec!(2500)
    }
}

/// Insider activity score and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsiderActivityScore {
    /// Overall insider activity score (0-100)
    pub score: Decimal,
    /// Net insider buying/selling direction
    pub direction: ChangeDirection,
    /// Total insider buying in shares
    pub total_buying: i64,
    /// Total insider selling in shares
    pub total_selling: i64,
    /// Net change in shares
    pub net_change: i64,
    /// Number of insider transactions
    pub transaction_count: usize,
    /// Significant insider buys (> 0.5% change)
    pub significant_buys: Vec<String>,
    /// Significant insider sells (> 0.5% change)
    pub significant_sells: Vec<String>,
}

impl InsiderActivityScore {
    /// Calculate insider activity score from ownership changes
    pub fn from_changes(changes: &[OwnershipChange]) -> Self {
        let insider_changes: Vec<&OwnershipChange> = changes
            .iter()
            .filter(|c| matches!(c.shareholder_type, ShareholderType::Insider))
            .collect();

        let total_buying: i64 = insider_changes
            .iter()
            .filter(|c| c.change_shares > 0)
            .map(|c| c.change_shares)
            .sum();

        let total_selling: i64 = insider_changes
            .iter()
            .filter(|c| c.change_shares < 0)
            .map(|c| -c.change_shares)
            .sum();

        let net_change = total_buying - total_selling;

        let direction = if net_change > 0 {
            ChangeDirection::Increase
        } else if net_change < 0 {
            ChangeDirection::Decrease
        } else {
            ChangeDirection::NoChange
        };

        let significant_buys: Vec<String> = insider_changes
            .iter()
            .filter(|c| c.is_significant && c.change_shares > 0)
            .map(|c| c.shareholder_name.clone())
            .collect();

        let significant_sells: Vec<String> = insider_changes
            .iter()
            .filter(|c| c.is_significant && c.change_shares < 0)
            .map(|c| c.shareholder_name.clone())
            .collect();

        // Score calculation:
        // Base: 50 (neutral)
        // +25 max for net buying
        // -25 max for net selling
        // +15 for significant buys by multiple insiders
        // -10 for significant sells by multiple insiders
        let mut score = dec!(50);

        if total_buying + total_selling > 0 {
            let buy_ratio =
                Decimal::from(total_buying) / Decimal::from(total_buying + total_selling);
            score += (buy_ratio - dec!(0.5)) * dec!(50); // -25 to +25
        }

        // Bonus for conviction (multiple significant transactions)
        if significant_buys.len() >= 2 {
            score += dec!(15);
        } else if !significant_buys.is_empty() {
            score += dec!(8);
        }

        if significant_sells.len() >= 2 {
            score -= dec!(10);
        } else if !significant_sells.is_empty() {
            score -= dec!(5);
        }

        // Clamp to 0-100
        score = score.max(Decimal::ZERO).min(dec!(100));

        Self {
            score,
            direction,
            total_buying,
            total_selling,
            net_change,
            transaction_count: insider_changes.len(),
            significant_buys,
            significant_sells,
        }
    }

    /// Check if insider activity is bullish
    pub fn is_bullish(&self) -> bool {
        self.score > dec!(60) && matches!(self.direction, ChangeDirection::Increase)
    }

    /// Check if insider activity is bearish
    pub fn is_bearish(&self) -> bool {
        self.score < dec!(40) && matches!(self.direction, ChangeDirection::Decrease)
    }
}

/// Institutional flow analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionalFlow {
    /// Net institutional shares change
    pub net_shares: i64,
    /// Net institutional percentage change
    pub net_percentage: Decimal,
    /// Institutions increasing positions
    pub accumulators: Vec<String>,
    /// Institutions decreasing positions
    pub distributors: Vec<String>,
    /// New institutional investors
    pub new_entrants: Vec<String>,
    /// Institutions that fully exited
    pub exits: Vec<String>,
}

impl InstitutionalFlow {
    /// Calculate institutional flow from ownership changes
    pub fn from_changes(changes: &[OwnershipChange]) -> Self {
        let institutional_changes: Vec<&OwnershipChange> = changes
            .iter()
            .filter(|c| matches!(c.shareholder_type, ShareholderType::Institution))
            .collect();

        let net_shares: i64 = institutional_changes.iter().map(|c| c.change_shares).sum();

        let net_percentage: Decimal = institutional_changes
            .iter()
            .map(|c| c.change_percentage)
            .sum();

        let accumulators: Vec<String> = institutional_changes
            .iter()
            .filter(|c| c.change_shares > 0 && c.previous_shares > 0)
            .map(|c| c.shareholder_name.clone())
            .collect();

        let distributors: Vec<String> = institutional_changes
            .iter()
            .filter(|c| c.change_shares < 0 && c.current_shares > 0)
            .map(|c| c.shareholder_name.clone())
            .collect();

        let new_entrants: Vec<String> = institutional_changes
            .iter()
            .filter(|c| c.previous_shares == 0 && c.current_shares > 0)
            .map(|c| c.shareholder_name.clone())
            .collect();

        let exits: Vec<String> = institutional_changes
            .iter()
            .filter(|c| c.previous_shares > 0 && c.current_shares == 0)
            .map(|c| c.shareholder_name.clone())
            .collect();

        Self {
            net_shares,
            net_percentage,
            accumulators,
            distributors,
            new_entrants,
            exits,
        }
    }

    /// Check if there's net institutional accumulation
    pub fn is_accumulating(&self) -> bool {
        self.net_shares > 0
    }

    /// Check if there's net institutional distribution
    pub fn is_distributing(&self) -> bool {
        self.net_shares < 0
    }
}

/// Overall shareholding health score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareholdingScore {
    /// Overall score (0-100)
    pub score: Decimal,
    /// Insider activity component
    pub insider_score: Decimal,
    /// Institutional flow component
    pub institutional_score: Decimal,
    /// Concentration component (higher free float = better)
    pub liquidity_score: Decimal,
    /// Key insights
    pub insights: Vec<String>,
}

impl ShareholdingScore {
    /// Calculate overall shareholding score
    pub fn calculate(snapshot: &ShareholdingSnapshot, changes: &[OwnershipChange]) -> Self {
        let concentration = ConcentrationMetrics::from_snapshot(snapshot);
        let insider_activity = InsiderActivityScore::from_changes(changes);
        let institutional_flow = InstitutionalFlow::from_changes(changes);

        // Liquidity score: higher free float = better (max 100 at 50%+ free float)
        let liquidity_score = (concentration.free_float * dec!(2)).min(dec!(100));

        // Institutional score: based on net flow direction
        let mut institutional_score = dec!(50);
        if institutional_flow.is_accumulating() {
            institutional_score += dec!(25);
            if !institutional_flow.new_entrants.is_empty() {
                institutional_score += dec!(10);
            }
        } else if institutional_flow.is_distributing() {
            institutional_score -= dec!(20);
            if !institutional_flow.exits.is_empty() {
                institutional_score -= dec!(10);
            }
        }
        institutional_score = institutional_score.max(Decimal::ZERO).min(dec!(100));

        // Weighted average
        let score = (insider_activity.score * dec!(0.35)
            + institutional_score * dec!(0.35)
            + liquidity_score * dec!(0.30))
        .round_dp(2);

        // Generate insights
        let mut insights = Vec::new();

        if insider_activity.is_bullish() {
            insights.push("Strong insider buying detected".to_string());
        } else if insider_activity.is_bearish() {
            insights.push("Insider selling pressure detected".to_string());
        }

        if institutional_flow.is_accumulating() {
            insights.push(format!(
                "{} institutions accumulating",
                institutional_flow.accumulators.len()
            ));
        } else if institutional_flow.is_distributing() {
            insights.push(format!(
                "{} institutions distributing",
                institutional_flow.distributors.len()
            ));
        }

        if concentration.is_highly_concentrated() {
            insights.push("Highly concentrated ownership - lower liquidity".to_string());
        }

        if concentration.free_float < dec!(20) {
            insights.push("Low free float - limited public trading".to_string());
        }

        Self {
            score,
            insider_score: insider_activity.score,
            institutional_score,
            liquidity_score,
            insights,
        }
    }
}

/// Detect ownership accumulation pattern
///
/// Returns true if there's a consistent pattern of accumulation
/// by insiders or institutions over multiple periods.
pub fn detect_accumulation_pattern(snapshots: &[ShareholdingSnapshot]) -> bool {
    if snapshots.len() < 2 {
        return false;
    }

    let mut consecutive_increases = 0;

    for window in snapshots.windows(2) {
        let prev = &window[0];
        let curr = &window[1];

        // Check if insider + institutional ownership increased
        let prev_ownership = prev.insider_ownership + prev.institutional_ownership;
        let curr_ownership = curr.insider_ownership + curr.institutional_ownership;

        if curr_ownership > prev_ownership {
            consecutive_increases += 1;
        } else {
            consecutive_increases = 0;
        }
    }

    // At least 2 consecutive periods of accumulation
    consecutive_increases >= 2
}

/// Detect distribution pattern (smart money selling)
pub fn detect_distribution_pattern(snapshots: &[ShareholdingSnapshot]) -> bool {
    if snapshots.len() < 2 {
        return false;
    }

    let mut consecutive_decreases = 0;

    for window in snapshots.windows(2) {
        let prev = &window[0];
        let curr = &window[1];

        let prev_ownership = prev.insider_ownership + prev.institutional_ownership;
        let curr_ownership = curr.insider_ownership + curr.institutional_ownership;

        if curr_ownership < prev_ownership {
            consecutive_decreases += 1;
        } else {
            consecutive_decreases = 0;
        }
    }

    consecutive_decreases >= 2
}

#[cfg(test)]
mod tests {
    use super::super::models::Shareholder;
    use super::*;
    use chrono::NaiveDate;

    fn make_shareholder(name: &str, stype: ShareholderType, shares: i64, pct: i32) -> Shareholder {
        Shareholder::with_type(name.to_string(), stype, shares, Decimal::from(pct))
    }

    fn make_snapshot(
        shareholders: Vec<Shareholder>,
        insider_pct: i32,
        inst_pct: i32,
        free_float: i32,
    ) -> ShareholdingSnapshot {
        ShareholdingSnapshot {
            symbol: "TEST".to_string(),
            report_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            total_shares: 100_000_000,
            shareholders,
            free_float: Decimal::from(free_float),
            insider_ownership: Decimal::from(insider_pct),
            institutional_ownership: Decimal::from(inst_pct),
            top_5_concentration: Decimal::from(80),
        }
    }

    #[test]
    fn test_concentration_metrics() {
        let snapshot = ShareholdingSnapshot::new(
            "TEST".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            100_000_000,
            vec![
                make_shareholder("Owner A", ShareholderType::Insider, 30_000_000, 30),
                make_shareholder("Owner B", ShareholderType::Institution, 25_000_000, 25),
                make_shareholder("Owner C", ShareholderType::Institution, 15_000_000, 15),
                make_shareholder("Public", ShareholderType::Public, 30_000_000, 30),
            ],
        );

        let metrics = ConcentrationMetrics::from_snapshot(&snapshot);

        assert_eq!(metrics.top_1_percentage, Decimal::from(30));
        // Two shareholders have 30%, so top 3 = 30+30+25=85
        assert_eq!(metrics.top_3_percentage, Decimal::from(85));
        assert!(metrics.hhi > dec!(0)); // Should have some concentration
    }

    #[test]
    fn test_insider_activity_bullish() {
        let changes = vec![
            OwnershipChange::from_snapshots(
                "TEST",
                "CEO",
                ShareholderType::Insider,
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                100_000,
                200_000,
                Decimal::from(1),
                Decimal::from(2),
            ),
            OwnershipChange::from_snapshots(
                "TEST",
                "CFO",
                ShareholderType::Insider,
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                50_000,
                100_000,
                Decimal::new(5, 1),
                Decimal::from(1),
            ),
        ];

        let score = InsiderActivityScore::from_changes(&changes);

        assert!(score.is_bullish());
        assert_eq!(score.direction, ChangeDirection::Increase);
        assert!(score.score > dec!(60));
    }

    #[test]
    fn test_institutional_flow_accumulation() {
        let changes = vec![
            OwnershipChange::from_snapshots(
                "TEST",
                "Fund A",
                ShareholderType::Institution,
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                1_000_000,
                1_500_000,
                Decimal::from(5),
                Decimal::new(75, 1),
            ),
            OwnershipChange::from_snapshots(
                "TEST",
                "Fund B",
                ShareholderType::Institution,
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                0,
                500_000, // New entrant
                Decimal::ZERO,
                Decimal::new(25, 1),
            ),
        ];

        let flow = InstitutionalFlow::from_changes(&changes);

        assert!(flow.is_accumulating());
        assert_eq!(flow.accumulators.len(), 1);
        assert_eq!(flow.new_entrants.len(), 1);
    }

    #[test]
    fn test_detect_accumulation_pattern() {
        let snapshots = vec![
            make_snapshot(vec![], 10, 20, 70),
            make_snapshot(vec![], 12, 22, 66), // +4% smart money
            make_snapshot(vec![], 14, 25, 61), // +5% smart money
        ];

        assert!(detect_accumulation_pattern(&snapshots));
    }

    #[test]
    fn test_detect_distribution_pattern() {
        let snapshots = vec![
            make_snapshot(vec![], 20, 30, 50),
            make_snapshot(vec![], 18, 28, 54), // -4% smart money
            make_snapshot(vec![], 15, 25, 60), // -6% smart money
        ];

        assert!(detect_distribution_pattern(&snapshots));
    }

    #[test]
    fn test_shareholding_score() {
        let snapshot = make_snapshot(
            vec![
                make_shareholder("Insider", ShareholderType::Insider, 20_000_000, 20),
                make_shareholder("Fund", ShareholderType::Institution, 30_000_000, 30),
                make_shareholder("Public", ShareholderType::Public, 50_000_000, 50),
            ],
            20,
            30,
            50,
        );

        let changes = vec![OwnershipChange::from_snapshots(
            "TEST",
            "Fund",
            ShareholderType::Institution,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            25_000_000,
            30_000_000,
            Decimal::from(25),
            Decimal::from(30),
        )];

        let score = ShareholdingScore::calculate(&snapshot, &changes);

        assert!(score.score > dec!(50)); // Should be positive due to institutional accumulation
        assert!(score.liquidity_score > dec!(90)); // 50% free float is good
    }
}
