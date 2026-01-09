//! Sector peer comparison

use crate::metrics::ValuationRatios;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Sector average metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorAverages {
    pub sector: String,
    pub avg_pe: Option<Decimal>,
    pub avg_pb: Option<Decimal>,
    pub avg_ev_ebitda: Option<Decimal>,
    pub avg_roe: Option<Decimal>,
    pub avg_profit_margin: Option<Decimal>,
    pub peer_count: usize,
}

/// Peer comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerComparison {
    pub symbol: String,
    pub sector: String,
    pub pe_percentile: Option<Decimal>,
    pub pb_percentile: Option<Decimal>,
    pub ev_ebitda_percentile: Option<Decimal>,
    pub roe_percentile: Option<Decimal>,
    pub overall_rank: usize,
    pub total_peers: usize,
}

/// Calculate sector averages from peer ratios
pub fn calculate_sector_averages(sector: &str, peer_ratios: &[ValuationRatios]) -> SectorAverages {
    let count = peer_ratios.len();

    if count == 0 {
        return SectorAverages {
            sector: sector.to_string(),
            avg_pe: None,
            avg_pb: None,
            avg_ev_ebitda: None,
            avg_roe: None,
            avg_profit_margin: None,
            peer_count: 0,
        };
    }

    let avg_pe = calculate_average(peer_ratios.iter().filter_map(|r| r.pe_ratio).collect());
    let avg_pb = calculate_average(peer_ratios.iter().filter_map(|r| r.pb_ratio).collect());
    let avg_ev_ebitda =
        calculate_average(peer_ratios.iter().filter_map(|r| r.ev_ebitda).collect());
    let avg_roe = calculate_average(peer_ratios.iter().filter_map(|r| r.roe).collect());
    let avg_profit_margin =
        calculate_average(peer_ratios.iter().filter_map(|r| r.profit_margin).collect());

    SectorAverages {
        sector: sector.to_string(),
        avg_pe,
        avg_pb,
        avg_ev_ebitda,
        avg_roe,
        avg_profit_margin,
        peer_count: count,
    }
}

/// Calculate average of decimal values
fn calculate_average(values: Vec<Decimal>) -> Option<Decimal> {
    if values.is_empty() {
        return None;
    }
    let sum: Decimal = values.iter().sum();
    Some((sum / Decimal::from(values.len() as i64)).round_dp(2))
}

/// Calculate percentile rank (lower is better for P/E, P/B, EV/EBITDA)
pub fn calculate_percentile(value: Decimal, all_values: &[Decimal], lower_is_better: bool) -> Decimal {
    if all_values.is_empty() {
        return dec!(50);
    }

    let count_below = all_values.iter().filter(|v| **v < value).count();
    let percentile =
        Decimal::from(count_below as i64) / Decimal::from(all_values.len() as i64) * dec!(100);

    if lower_is_better {
        dec!(100) - percentile // Invert so lower values get higher percentile
    } else {
        percentile
    }
}

/// Compare stock against peers
pub fn compare_to_peers(
    symbol: &str,
    ratios: &ValuationRatios,
    sector: &str,
    peer_ratios: &[(String, ValuationRatios)],
) -> PeerComparison {
    let total_peers = peer_ratios.len();

    // Collect peer values for percentile calculation
    let pe_values: Vec<Decimal> = peer_ratios
        .iter()
        .filter_map(|(_, r)| r.pe_ratio)
        .collect();
    let pb_values: Vec<Decimal> = peer_ratios
        .iter()
        .filter_map(|(_, r)| r.pb_ratio)
        .collect();
    let ev_values: Vec<Decimal> = peer_ratios
        .iter()
        .filter_map(|(_, r)| r.ev_ebitda)
        .collect();
    let roe_values: Vec<Decimal> = peer_ratios.iter().filter_map(|(_, r)| r.roe).collect();

    let pe_percentile = ratios
        .pe_ratio
        .map(|pe| calculate_percentile(pe, &pe_values, true));
    let pb_percentile = ratios
        .pb_ratio
        .map(|pb| calculate_percentile(pb, &pb_values, true));
    let ev_ebitda_percentile = ratios
        .ev_ebitda
        .map(|ev| calculate_percentile(ev, &ev_values, true));
    let roe_percentile = ratios
        .roe
        .map(|roe| calculate_percentile(roe, &roe_values, false)); // Higher ROE is better

    // Calculate overall rank based on average percentile
    let percentiles: Vec<Decimal> = [
        pe_percentile,
        pb_percentile,
        ev_ebitda_percentile,
        roe_percentile,
    ]
    .into_iter()
    .flatten()
    .collect();

    let avg_percentile = if !percentiles.is_empty() {
        percentiles.iter().sum::<Decimal>() / Decimal::from(percentiles.len() as i64)
    } else {
        dec!(50)
    };

    // Convert percentile to rank
    let overall_rank = ((dec!(100) - avg_percentile) / dec!(100)
        * Decimal::from(total_peers as i64))
    .to_string()
    .parse::<f64>()
    .unwrap_or(0.0) as usize
        + 1;

    PeerComparison {
        symbol: symbol.to_string(),
        sector: sector.to_string(),
        pe_percentile,
        pb_percentile,
        ev_ebitda_percentile,
        roe_percentile,
        overall_rank: overall_rank.min(total_peers).max(1),
        total_peers,
    }
}

/// IDX sector classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IdxSector {
    Banking,
    Finance,
    Telco,
    Consumer,
    Infrastructure,
    Mining,
    Energy,
    Property,
    Healthcare,
    Technology,
    Industrial,
    BasicMaterials,
    Other,
}

impl IdxSector {
    /// Get typical P/E range for sector
    pub fn typical_pe_range(&self) -> (Decimal, Decimal) {
        match self {
            IdxSector::Banking => (dec!(8), dec!(15)),
            IdxSector::Finance => (dec!(10), dec!(20)),
            IdxSector::Telco => (dec!(12), dec!(20)),
            IdxSector::Consumer => (dec!(15), dec!(30)),
            IdxSector::Infrastructure => (dec!(10), dec!(18)),
            IdxSector::Mining => (dec!(5), dec!(12)),
            IdxSector::Energy => (dec!(6), dec!(15)),
            IdxSector::Property => (dec!(8), dec!(15)),
            IdxSector::Healthcare => (dec!(20), dec!(40)),
            IdxSector::Technology => (dec!(20), dec!(50)),
            IdxSector::Industrial => (dec!(10), dec!(20)),
            IdxSector::BasicMaterials => (dec!(8), dec!(15)),
            IdxSector::Other => (dec!(10), dec!(20)),
        }
    }

    /// Get typical EV/EBITDA range for sector
    pub fn typical_ev_ebitda_range(&self) -> (Decimal, Decimal) {
        match self {
            IdxSector::Banking => (dec!(5), dec!(10)),
            IdxSector::Finance => (dec!(6), dec!(12)),
            IdxSector::Telco => (dec!(5), dec!(8)),
            IdxSector::Consumer => (dec!(8), dec!(15)),
            IdxSector::Infrastructure => (dec!(6), dec!(12)),
            IdxSector::Mining => (dec!(3), dec!(8)),
            IdxSector::Energy => (dec!(4), dec!(10)),
            IdxSector::Property => (dec!(8), dec!(15)),
            IdxSector::Healthcare => (dec!(10), dec!(20)),
            IdxSector::Technology => (dec!(12), dec!(25)),
            IdxSector::Industrial => (dec!(6), dec!(12)),
            IdxSector::BasicMaterials => (dec!(5), dec!(10)),
            IdxSector::Other => (dec!(6), dec!(12)),
        }
    }

    /// Classify from sector string
    pub fn from_sector_name(name: &str) -> Self {
        let lower = name.to_lowercase();
        if lower.contains("bank") {
            IdxSector::Banking
        } else if lower.contains("financ") || lower.contains("insurance") {
            IdxSector::Finance
        } else if lower.contains("telco") || lower.contains("telecom") {
            IdxSector::Telco
        } else if lower.contains("consumer") || lower.contains("retail") || lower.contains("food") {
            IdxSector::Consumer
        } else if lower.contains("infra")
            || lower.contains("construction")
            || lower.contains("toll")
        {
            IdxSector::Infrastructure
        } else if lower.contains("mining") || lower.contains("coal") || lower.contains("nickel") {
            IdxSector::Mining
        } else if lower.contains("energy") || lower.contains("oil") || lower.contains("gas") {
            IdxSector::Energy
        } else if lower.contains("property") || lower.contains("real estate") {
            IdxSector::Property
        } else if lower.contains("health") || lower.contains("pharma") || lower.contains("hospital")
        {
            IdxSector::Healthcare
        } else if lower.contains("tech") || lower.contains("software") || lower.contains("digital")
        {
            IdxSector::Technology
        } else if lower.contains("industrial") || lower.contains("manufactur") {
            IdxSector::Industrial
        } else if lower.contains("basic") || lower.contains("chemical") || lower.contains("cement")
        {
            IdxSector::BasicMaterials
        } else {
            IdxSector::Other
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sector_averages() {
        let ratios = vec![
            ValuationRatios {
                pe_ratio: Some(dec!(10)),
                pb_ratio: Some(dec!(1)),
                ev_ebitda: Some(dec!(8)),
                roe: Some(dec!(15)),
                ..Default::default()
            },
            ValuationRatios {
                pe_ratio: Some(dec!(20)),
                pb_ratio: Some(dec!(2)),
                ev_ebitda: Some(dec!(12)),
                roe: Some(dec!(20)),
                ..Default::default()
            },
        ];

        let avg = calculate_sector_averages("Banking", &ratios);

        assert_eq!(avg.avg_pe, Some(dec!(15)));
        assert_eq!(avg.avg_pb, Some(dec!(1.5)));
        assert_eq!(avg.peer_count, 2);
    }

    #[test]
    fn test_percentile() {
        let values = vec![dec!(5), dec!(10), dec!(15), dec!(20), dec!(25)];

        // For P/E (lower is better), value of 10 should be in high percentile
        let pct = calculate_percentile(dec!(10), &values, true);
        assert!(pct > dec!(50)); // Should be above average
    }

    #[test]
    fn test_sector_classification() {
        assert_eq!(IdxSector::from_sector_name("Banking"), IdxSector::Banking);
        assert_eq!(IdxSector::from_sector_name("Coal Mining"), IdxSector::Mining);
        assert_eq!(
            IdxSector::from_sector_name("Telecommunications"),
            IdxSector::Telco
        );
    }
}
