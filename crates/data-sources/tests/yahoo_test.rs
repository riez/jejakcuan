//! Integration tests for Yahoo Finance client
//! Run with: cargo test -p jejakcuan-data-sources --test yahoo_test -- --ignored

use jejakcuan_data_sources::YahooFinanceClient;

#[tokio::test]
#[ignore] // Ignored by default - requires network
async fn test_get_quote() {
    let client = YahooFinanceClient::new();
    let quote = client.get_quote("BBCA").await.expect("Failed to get quote");

    assert_eq!(quote.symbol, "BBCA.JK");
    assert!(quote.regular_market_price.is_some());
    println!("BBCA price: {:?}", quote.regular_market_price);
}

#[tokio::test]
#[ignore]
async fn test_get_history() {
    let client = YahooFinanceClient::new();
    let history = client
        .get_daily_history_1y("BBCA")
        .await
        .expect("Failed to get history");

    assert!(!history.is_empty());
    println!("Got {} data points for BBCA", history.len());

    // Check first and last data point
    if let Some(first) = history.first() {
        println!(
            "First: {} - O:{} H:{} L:{} C:{} V:{}",
            first.timestamp, first.open, first.high, first.low, first.close, first.volume
        );
    }
}

#[tokio::test]
#[ignore]
async fn test_get_multiple_quotes() {
    let client = YahooFinanceClient::new();
    let quotes = client
        .get_quotes(&["BBCA", "TLKM", "ASII"])
        .await
        .expect("Failed to get quotes");

    assert_eq!(quotes.len(), 3);
    for quote in &quotes {
        println!("{}: {:?}", quote.symbol, quote.regular_market_price);
    }
}
