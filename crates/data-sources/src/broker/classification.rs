//! Broker classification by category

use super::models::BrokerCategory;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Static broker classification map
static BROKER_CLASSIFICATIONS: LazyLock<HashMap<&'static str, BrokerCategory>> =
    LazyLock::new(|| {
        let mut map = HashMap::new();

        // Foreign Institutional
        map.insert("BK", BrokerCategory::ForeignInstitutional); // JP Morgan
        map.insert("KZ", BrokerCategory::ForeignInstitutional); // CLSA
        map.insert("CS", BrokerCategory::ForeignInstitutional); // Credit Suisse
        map.insert("AK", BrokerCategory::ForeignInstitutional); // UBS
        map.insert("GW", BrokerCategory::ForeignInstitutional); // HSBC
        map.insert("DP", BrokerCategory::ForeignInstitutional); // DBS Vickers
        map.insert("RX", BrokerCategory::ForeignInstitutional); // Macquarie
        map.insert("ZP", BrokerCategory::ForeignInstitutional); // Maybank
        map.insert("ML", BrokerCategory::ForeignInstitutional); // Merrill Lynch
        map.insert("DB", BrokerCategory::ForeignInstitutional); // Deutsche Bank

        // Local Institutional
        map.insert("CC", BrokerCategory::LocalInstitutional); // Mandiri Sekuritas
        map.insert("SQ", BrokerCategory::LocalInstitutional); // BCA Sekuritas
        map.insert("NI", BrokerCategory::LocalInstitutional); // BNI Sekuritas
        map.insert("OD", BrokerCategory::LocalInstitutional); // BRI Danareksa
        map.insert("HP", BrokerCategory::LocalInstitutional); // Henan Putihrai
        map.insert("KI", BrokerCategory::LocalInstitutional); // Ciptadana
        map.insert("DX", BrokerCategory::LocalInstitutional); // Bahana
        map.insert("IF", BrokerCategory::LocalInstitutional); // Samuel
        map.insert("LG", BrokerCategory::LocalInstitutional); // Trimegah
        map.insert("PD", BrokerCategory::LocalInstitutional); // Indo Premier
        map.insert("YU", BrokerCategory::LocalInstitutional); // CGS-CIMB
        map.insert("MS", BrokerCategory::LocalInstitutional); // Morgan Stanley

        // Retail-focused brokers
        map.insert("EP", BrokerCategory::Retail); // MNC Sekuritas
        map.insert("AI", BrokerCategory::Retail); // Ajaib
        map.insert("GR", BrokerCategory::Retail); // Mirae Asset
        map.insert("AG", BrokerCategory::Retail); // Artha Sekuritas
        map.insert("PS", BrokerCategory::Retail); // Panin Sekuritas
        map.insert("TP", BrokerCategory::Retail); // Toko
        map.insert("BI", BrokerCategory::Retail); // Bibit (PT Bibit Tumbuh Bersama)

        map
    });

/// Get broker category from code
pub fn get_broker_category(code: &str) -> BrokerCategory {
    BROKER_CLASSIFICATIONS
        .get(code.to_uppercase().as_str())
        .copied()
        .unwrap_or(BrokerCategory::Unknown)
}

/// Check if broker is foreign institutional
pub fn is_foreign_broker(code: &str) -> bool {
    matches!(
        get_broker_category(code),
        BrokerCategory::ForeignInstitutional
    )
}

/// Check if broker is institutional (foreign or local)
pub fn is_institutional_broker(code: &str) -> bool {
    matches!(
        get_broker_category(code),
        BrokerCategory::ForeignInstitutional | BrokerCategory::LocalInstitutional
    )
}

/// Get all broker codes for a category
pub fn get_brokers_by_category(category: BrokerCategory) -> Vec<&'static str> {
    BROKER_CLASSIFICATIONS
        .iter()
        .filter(|(_, cat)| **cat == category)
        .map(|(code, _)| *code)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_broker_classification() {
        assert_eq!(
            get_broker_category("BK"),
            BrokerCategory::ForeignInstitutional
        );
        assert_eq!(
            get_broker_category("CC"),
            BrokerCategory::LocalInstitutional
        );
        assert_eq!(get_broker_category("EP"), BrokerCategory::Retail);
        assert_eq!(get_broker_category("XX"), BrokerCategory::Unknown);
    }

    #[test]
    fn test_is_institutional() {
        assert!(is_institutional_broker("BK"));
        assert!(is_institutional_broker("CC"));
        assert!(!is_institutional_broker("EP"));
    }
}
