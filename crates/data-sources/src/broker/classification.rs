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

    #[test]
    fn test_is_foreign_broker() {
        // Foreign institutional brokers
        assert!(is_foreign_broker("BK")); // JP Morgan
        assert!(is_foreign_broker("KZ")); // CLSA
        assert!(is_foreign_broker("CS")); // Credit Suisse
        assert!(is_foreign_broker("ML")); // Merrill Lynch
        assert!(is_foreign_broker("DB")); // Deutsche Bank

        // Local institutional - not foreign
        assert!(!is_foreign_broker("CC")); // Mandiri Sekuritas
        assert!(!is_foreign_broker("SQ")); // BCA Sekuritas

        // Retail - not foreign
        assert!(!is_foreign_broker("EP")); // MNC Sekuritas
        assert!(!is_foreign_broker("AI")); // Ajaib

        // Unknown - not foreign
        assert!(!is_foreign_broker("XX"));
    }

    #[test]
    fn test_case_insensitivity() {
        // Should handle lowercase
        assert_eq!(
            get_broker_category("bk"),
            BrokerCategory::ForeignInstitutional
        );
        // Should handle mixed case
        assert_eq!(
            get_broker_category("Bk"),
            BrokerCategory::ForeignInstitutional
        );
    }

    #[test]
    fn test_all_foreign_brokers() {
        let foreign_codes = ["BK", "KZ", "CS", "AK", "GW", "DP", "RX", "ZP", "ML", "DB"];
        for code in &foreign_codes {
            assert!(
                is_foreign_broker(code),
                "Expected {} to be foreign broker",
                code
            );
            assert_eq!(
                get_broker_category(code),
                BrokerCategory::ForeignInstitutional,
                "Expected {} to be ForeignInstitutional",
                code
            );
        }
    }

    #[test]
    fn test_all_local_institutional_brokers() {
        let local_codes = [
            "CC", "SQ", "NI", "OD", "HP", "KI", "DX", "IF", "LG", "PD", "YU", "MS",
        ];
        for code in &local_codes {
            assert!(
                is_institutional_broker(code),
                "Expected {} to be institutional",
                code
            );
            assert_eq!(
                get_broker_category(code),
                BrokerCategory::LocalInstitutional,
                "Expected {} to be LocalInstitutional",
                code
            );
        }
    }

    #[test]
    fn test_all_retail_brokers() {
        let retail_codes = ["EP", "AI", "GR", "AG", "PS", "TP", "BI"];
        for code in &retail_codes {
            assert!(
                !is_institutional_broker(code),
                "Expected {} to not be institutional",
                code
            );
            assert_eq!(
                get_broker_category(code),
                BrokerCategory::Retail,
                "Expected {} to be Retail",
                code
            );
        }
    }

    #[test]
    fn test_get_brokers_by_category() {
        let foreign_brokers = get_brokers_by_category(BrokerCategory::ForeignInstitutional);
        assert!(foreign_brokers.contains(&"BK"));
        assert!(foreign_brokers.contains(&"ML"));
        assert!(!foreign_brokers.contains(&"CC")); // Local

        let local_brokers = get_brokers_by_category(BrokerCategory::LocalInstitutional);
        assert!(local_brokers.contains(&"CC"));
        assert!(local_brokers.contains(&"SQ"));
        assert!(!local_brokers.contains(&"BK")); // Foreign

        let retail_brokers = get_brokers_by_category(BrokerCategory::Retail);
        assert!(retail_brokers.contains(&"EP"));
        assert!(retail_brokers.contains(&"AI"));
    }

    #[test]
    fn test_unknown_broker() {
        let unknown_codes = ["XX", "YY", "ZZ", "00", "ABCD"];
        for code in &unknown_codes {
            assert_eq!(
                get_broker_category(code),
                BrokerCategory::Unknown,
                "Expected {} to be Unknown",
                code
            );
            assert!(
                !is_foreign_broker(code),
                "Expected {} to not be foreign",
                code
            );
            assert!(
                !is_institutional_broker(code),
                "Expected {} to not be institutional",
                code
            );
        }
    }

    #[test]
    fn test_broker_category_count() {
        // Verify we have the expected number of brokers in each category
        let foreign = get_brokers_by_category(BrokerCategory::ForeignInstitutional);
        assert_eq!(foreign.len(), 10);

        let local = get_brokers_by_category(BrokerCategory::LocalInstitutional);
        assert_eq!(local.len(), 12);

        let retail = get_brokers_by_category(BrokerCategory::Retail);
        assert_eq!(retail.len(), 7);
    }
}
