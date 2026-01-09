//! Shareholding data models
//!
//! Data structures for representing shareholding information from KSEI/OJK.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Type of shareholder
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShareholderType {
    /// Company insider (director, commissioner)
    Insider,
    /// Institutional investor
    Institution,
    /// Public/retail investors
    Public,
    /// Government entity
    Government,
    /// Unknown/other
    Other,
}

impl ShareholderType {
    /// Infer shareholder type from name patterns
    pub fn from_name(name: &str) -> Self {
        let lower = name.to_lowercase();

        if lower.contains("direktur")
            || lower.contains("komisaris")
            || lower.contains("director")
            || lower.contains("commissioner")
        {
            ShareholderType::Insider
        } else if lower.contains("negara")
            || lower.contains("pemerintah")
            || lower.contains("government")
        {
            ShareholderType::Government
        } else if lower.contains("bank")
            || lower.contains("fund")
            || lower.contains("capital")
            || lower.contains("investment")
            || lower.contains("aset")
            || lower.contains("sekuritas")
        {
            ShareholderType::Institution
        } else if lower.contains("publik") || lower.contains("public") {
            ShareholderType::Public
        } else {
            ShareholderType::Other
        }
    }

    /// Weight for ownership analysis scoring
    pub fn weight(&self) -> Decimal {
        match self {
            ShareholderType::Insider => Decimal::from(1),
            ShareholderType::Institution => Decimal::new(8, 1), // 0.8
            ShareholderType::Government => Decimal::new(6, 1),  // 0.6
            ShareholderType::Public => Decimal::new(3, 1),      // 0.3
            ShareholderType::Other => Decimal::new(5, 1),       // 0.5
        }
    }
}

/// Shareholder information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shareholder {
    /// Shareholder name
    pub name: String,
    /// Type of shareholder
    pub shareholder_type: ShareholderType,
    /// Number of shares held
    pub shares_held: i64,
    /// Percentage of total shares
    pub percentage: Decimal,
    /// Whether this is a company insider
    pub is_insider: bool,
}

impl Shareholder {
    /// Create a new shareholder with auto-detected type
    pub fn new(name: String, shares_held: i64, percentage: Decimal) -> Self {
        let shareholder_type = ShareholderType::from_name(&name);
        let is_insider = matches!(shareholder_type, ShareholderType::Insider);

        Self {
            name,
            shareholder_type,
            shares_held,
            percentage,
            is_insider,
        }
    }

    /// Create shareholder with explicit type
    pub fn with_type(
        name: String,
        shareholder_type: ShareholderType,
        shares_held: i64,
        percentage: Decimal,
    ) -> Self {
        let is_insider = matches!(shareholder_type, ShareholderType::Insider);

        Self {
            name,
            shareholder_type,
            shares_held,
            percentage,
            is_insider,
        }
    }
}

/// Shareholding snapshot for a stock at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareholdingSnapshot {
    /// Stock symbol
    pub symbol: String,
    /// Report date
    pub report_date: NaiveDate,
    /// Total outstanding shares
    pub total_shares: i64,
    /// List of major shareholders
    pub shareholders: Vec<Shareholder>,
    /// Free float percentage (publicly tradeable shares)
    pub free_float: Decimal,
    /// Total insider ownership percentage
    pub insider_ownership: Decimal,
    /// Total institutional ownership percentage
    pub institutional_ownership: Decimal,
    /// Top 5 shareholders concentration (sum of percentages)
    pub top_5_concentration: Decimal,
}

impl ShareholdingSnapshot {
    /// Create new snapshot with calculated metrics
    pub fn new(
        symbol: String,
        report_date: NaiveDate,
        total_shares: i64,
        shareholders: Vec<Shareholder>,
    ) -> Self {
        let insider_ownership: Decimal = shareholders
            .iter()
            .filter(|s| s.is_insider)
            .map(|s| s.percentage)
            .sum();

        let institutional_ownership: Decimal = shareholders
            .iter()
            .filter(|s| matches!(s.shareholder_type, ShareholderType::Institution))
            .map(|s| s.percentage)
            .sum();

        let mut sorted_by_percentage = shareholders.clone();
        sorted_by_percentage.sort_by(|a, b| b.percentage.cmp(&a.percentage));

        let top_5_concentration: Decimal = sorted_by_percentage
            .iter()
            .take(5)
            .map(|s| s.percentage)
            .sum();

        // Free float = 100% - (insider + government ownership)
        let non_free_float: Decimal = shareholders
            .iter()
            .filter(|s| {
                matches!(
                    s.shareholder_type,
                    ShareholderType::Insider | ShareholderType::Government
                )
            })
            .map(|s| s.percentage)
            .sum();

        let free_float = Decimal::from(100) - non_free_float;

        Self {
            symbol,
            report_date,
            total_shares,
            shareholders,
            free_float,
            insider_ownership,
            institutional_ownership,
            top_5_concentration,
        }
    }
}

/// Direction of ownership change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeDirection {
    /// Increase in ownership
    Increase,
    /// Decrease in ownership
    Decrease,
    /// No change
    NoChange,
}

/// Ownership change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipChange {
    /// Stock symbol
    pub symbol: String,
    /// Shareholder name
    pub shareholder_name: String,
    /// Type of shareholder
    pub shareholder_type: ShareholderType,
    /// Report date
    pub report_date: NaiveDate,
    /// Previous shares held
    pub previous_shares: i64,
    /// Current shares held
    pub current_shares: i64,
    /// Change in shares
    pub change_shares: i64,
    /// Previous ownership percentage
    pub previous_percentage: Decimal,
    /// Current ownership percentage
    pub current_percentage: Decimal,
    /// Change in percentage
    pub change_percentage: Decimal,
    /// Direction of change
    pub direction: ChangeDirection,
    /// Whether this is a significant change (> 1%)
    pub is_significant: bool,
}

impl OwnershipChange {
    /// Create ownership change from two snapshots
    #[allow(clippy::too_many_arguments)]
    pub fn from_snapshots(
        symbol: &str,
        shareholder_name: &str,
        shareholder_type: ShareholderType,
        report_date: NaiveDate,
        previous_shares: i64,
        current_shares: i64,
        previous_percentage: Decimal,
        current_percentage: Decimal,
    ) -> Self {
        let change_shares = current_shares - previous_shares;
        let change_percentage = current_percentage - previous_percentage;

        let direction = if change_shares > 0 {
            ChangeDirection::Increase
        } else if change_shares < 0 {
            ChangeDirection::Decrease
        } else {
            ChangeDirection::NoChange
        };

        // Significant if > 1% change
        let is_significant = change_percentage.abs() > Decimal::from(1);

        Self {
            symbol: symbol.to_string(),
            shareholder_name: shareholder_name.to_string(),
            shareholder_type,
            report_date,
            previous_shares,
            current_shares,
            change_shares,
            previous_percentage,
            current_percentage,
            change_percentage,
            direction,
            is_significant,
        }
    }
}

/// Data source for shareholding information
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShareholdingSource {
    /// KSEI (Kustodian Sentral Efek Indonesia)
    Ksei,
    /// OJK (Otoritas Jasa Keuangan)
    Ojk,
    /// IDX (Indonesia Stock Exchange)
    Idx,
    /// Manual entry
    Manual,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shareholder_type_from_name() {
        assert_eq!(
            ShareholderType::from_name("PT Bank Mandiri"),
            ShareholderType::Institution
        );
        assert_eq!(
            ShareholderType::from_name("Direktur Utama"),
            ShareholderType::Insider
        );
        assert_eq!(
            ShareholderType::from_name("Pemerintah RI"),
            ShareholderType::Government
        );
        assert_eq!(
            ShareholderType::from_name("Publik"),
            ShareholderType::Public
        );
        assert_eq!(
            ShareholderType::from_name("Random Name"),
            ShareholderType::Other
        );
    }

    #[test]
    fn test_shareholder_new() {
        let shareholder =
            Shareholder::new("PT Fund Manager".to_string(), 1_000_000, Decimal::from(5));

        assert_eq!(shareholder.shareholder_type, ShareholderType::Institution);
        assert!(!shareholder.is_insider);
    }

    #[test]
    fn test_shareholder_insider() {
        let shareholder =
            Shareholder::new("Komisaris Utama".to_string(), 100_000, Decimal::from(1));

        assert_eq!(shareholder.shareholder_type, ShareholderType::Insider);
        assert!(shareholder.is_insider);
    }

    #[test]
    fn test_shareholding_snapshot_metrics() {
        let shareholders = vec![
            Shareholder::with_type(
                "Insider A".to_string(),
                ShareholderType::Insider,
                10_000_000,
                Decimal::from(25),
            ),
            Shareholder::with_type(
                "Institution B".to_string(),
                ShareholderType::Institution,
                8_000_000,
                Decimal::from(20),
            ),
            Shareholder::with_type(
                "Government C".to_string(),
                ShareholderType::Government,
                6_000_000,
                Decimal::from(15),
            ),
            Shareholder::with_type(
                "Public".to_string(),
                ShareholderType::Public,
                16_000_000,
                Decimal::from(40),
            ),
        ];

        let snapshot = ShareholdingSnapshot::new(
            "BBCA".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            40_000_000,
            shareholders,
        );

        assert_eq!(snapshot.insider_ownership, Decimal::from(25));
        assert_eq!(snapshot.institutional_ownership, Decimal::from(20));
        assert_eq!(snapshot.top_5_concentration, Decimal::from(100)); // All 4 shareholders
        assert_eq!(snapshot.free_float, Decimal::from(60)); // 100 - 25 (insider) - 15 (gov)
    }

    #[test]
    fn test_ownership_change() {
        let change = OwnershipChange::from_snapshots(
            "BBCA",
            "Insider A",
            ShareholderType::Insider,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            1_000_000,
            1_500_000,
            Decimal::from(5),
            Decimal::new(75, 1), // 7.5%
        );

        assert_eq!(change.direction, ChangeDirection::Increase);
        assert_eq!(change.change_shares, 500_000);
        assert_eq!(change.change_percentage, Decimal::new(25, 1)); // 2.5%
        assert!(change.is_significant);
    }

    #[test]
    fn test_ownership_change_decrease() {
        let change = OwnershipChange::from_snapshots(
            "BBCA",
            "Institution B",
            ShareholderType::Institution,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            2_000_000,
            1_500_000,
            Decimal::from(10),
            Decimal::new(75, 1), // 7.5%
        );

        assert_eq!(change.direction, ChangeDirection::Decrease);
        assert_eq!(change.change_shares, -500_000);
        assert!(change.is_significant); // -2.5% change
    }
}
