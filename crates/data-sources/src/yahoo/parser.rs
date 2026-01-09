//! Yahoo Finance response parsers

use super::models::*;
use crate::error::DataSourceError;
use chrono::{TimeZone, Utc};
use rust_decimal::Decimal;
use std::str::FromStr;

/// Parse quote response into YahooQuote
pub fn parse_quote(value: &serde_json::Value) -> Result<YahooQuote, DataSourceError> {
    Ok(YahooQuote {
        symbol: value["symbol"].as_str().unwrap_or_default().to_string(),
        short_name: value["shortName"].as_str().map(String::from),
        long_name: value["longName"].as_str().map(String::from),
        regular_market_price: value["regularMarketPrice"].as_f64(),
        regular_market_change: value["regularMarketChange"].as_f64(),
        regular_market_change_percent: value["regularMarketChangePercent"].as_f64(),
        regular_market_volume: value["regularMarketVolume"].as_i64(),
        regular_market_open: value["regularMarketOpen"].as_f64(),
        regular_market_high: value["regularMarketDayHigh"].as_f64(),
        regular_market_low: value["regularMarketDayLow"].as_f64(),
        regular_market_previous_close: value["regularMarketPreviousClose"].as_f64(),
        market_cap: value["marketCap"].as_i64(),
        trailing_pe: value["trailingPE"].as_f64(),
        price_to_book: value["priceToBook"].as_f64(),
        fifty_two_week_high: value["fiftyTwoWeekHigh"].as_f64(),
        fifty_two_week_low: value["fiftyTwoWeekLow"].as_f64(),
    })
}

/// Parse chart response into OHLCV data
pub fn parse_chart(response: ChartResponse) -> Result<Vec<YahooOHLCV>, DataSourceError> {
    let result = response
        .chart
        .result
        .ok_or_else(|| DataSourceError::InvalidResponse("No chart data".to_string()))?;

    let data = result
        .first()
        .ok_or_else(|| DataSourceError::InvalidResponse("Empty chart result".to_string()))?;

    let quote = data
        .indicators
        .quote
        .first()
        .ok_or_else(|| DataSourceError::InvalidResponse("No quote indicators".to_string()))?;

    let adj_close = data
        .indicators
        .adj_close
        .as_ref()
        .and_then(|ac| ac.first())
        .map(|ac| &ac.adj_close);

    let mut ohlcv = Vec::new();

    for (i, &ts) in data.timestamp.iter().enumerate() {
        let open = quote.open.get(i).and_then(|v| *v);
        let high = quote.high.get(i).and_then(|v| *v);
        let low = quote.low.get(i).and_then(|v| *v);
        let close = quote.close.get(i).and_then(|v| *v);
        let volume = quote.volume.get(i).and_then(|v| *v);

        // Skip if any OHLC value is missing
        if let (Some(o), Some(h), Some(l), Some(c), Some(v)) = (open, high, low, close, volume) {
            let adj = adj_close
                .and_then(|ac| ac.get(i))
                .and_then(|v| *v)
                .and_then(|v| Decimal::from_str(&v.to_string()).ok());

            ohlcv.push(YahooOHLCV {
                timestamp: Utc.timestamp_opt(ts, 0).single().ok_or_else(|| {
                    DataSourceError::InvalidResponse("Invalid timestamp".to_string())
                })?,
                open: Decimal::from_str(&o.to_string()).unwrap_or_default(),
                high: Decimal::from_str(&h.to_string()).unwrap_or_default(),
                low: Decimal::from_str(&l.to_string()).unwrap_or_default(),
                close: Decimal::from_str(&c.to_string()).unwrap_or_default(),
                volume: v,
                adj_close: adj,
            });
        }
    }

    Ok(ohlcv)
}
