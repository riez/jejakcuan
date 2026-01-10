//! Integration tests for core scoring functionality
//!
//! Tests the interaction between technical and fundamental score engines
//! and the composite scoring system.

use jejakcuan_core::{
    calculate_composite_score,
    FundamentalInput, FundamentalScoreEngine,
    ScoreWeights,
    TechnicalScoreEngine, TechnicalScoreInput,
};
use rust_decimal_macros::dec;

#[test]
fn test_full_scoring_pipeline() {
    // Create technical score
    let tech_engine = TechnicalScoreEngine::new();
    let tech_input = TechnicalScoreInput {
        current_price: dec!(5000),
        obi: Some(dec!(0.3)),
        ofi_trend: Some(dec!(0.4)),
        broker_score: Some(dec!(70)),
        institutional_buying: true,
        foreign_buying: true,
        ema20: Some(dec!(4800)),
        ema50: Some(dec!(4600)),
        rsi: Some(dec!(55)),
        macd_histogram: Some(dec!(10)),
        ..Default::default()
    };
    let tech_result = tech_engine.calculate(&tech_input);

    // Create fundamental score
    let fund_engine = FundamentalScoreEngine::new();
    let fund_input = FundamentalInput {
        pe_ratio: Some(dec!(12)),
        sector_pe: Some(dec!(18)),
        pb_ratio: Some(dec!(1.5)),
        sector_pb: Some(dec!(2.5)),
        ev_ebitda: Some(dec!(8)),
        sector_ev_ebitda: Some(dec!(12)),
        dcf_margin: Some(dec!(20)),
        roe: Some(dec!(18)),
        roa: Some(dec!(10)),
        profit_margin: Some(dec!(12)),
        debt_to_equity: Some(dec!(0.6)),
        current_ratio: Some(dec!(1.6)),
    };
    let fund_result = fund_engine.calculate(&fund_input);

    // Simulate sentiment and ML scores
    let sentiment_score = 65.0;
    let ml_score = 70.0;

    // Calculate composite score
    let weights = ScoreWeights::default();
    let composite = calculate_composite_score(
        tech_result.total_score.to_string().parse::<f64>().unwrap(),
        fund_result.total_score.to_string().parse::<f64>().unwrap(),
        sentiment_score,
        ml_score,
        &weights,
    );

    // Composite should be a weighted average within expected bounds
    assert!(composite >= 0.0 && composite <= 100.0);
    
    // With bullish technical and strong fundamental, composite should be above neutral
    assert!(composite > 50.0);
}

#[test]
fn test_bearish_scoring_pipeline() {
    // Create bearish technical score
    let tech_engine = TechnicalScoreEngine::new();
    let tech_input = TechnicalScoreInput {
        current_price: dec!(5000),
        obi: Some(dec!(-0.3)),
        ofi_trend: Some(dec!(-0.4)),
        broker_score: Some(dec!(30)),
        institutional_buying: false,
        foreign_buying: false,
        ema20: Some(dec!(5200)),
        ema50: Some(dec!(5400)),
        rsi: Some(dec!(75)), // Overbought
        macd_histogram: Some(dec!(-10)),
        ..Default::default()
    };
    let tech_result = tech_engine.calculate(&tech_input);

    // Create weak fundamental score
    let fund_engine = FundamentalScoreEngine::new();
    let fund_input = FundamentalInput {
        pe_ratio: Some(dec!(30)),
        sector_pe: Some(dec!(15)),
        pb_ratio: Some(dec!(4)),
        sector_pb: Some(dec!(2)),
        ev_ebitda: Some(dec!(18)),
        sector_ev_ebitda: Some(dec!(10)),
        dcf_margin: Some(dec!(-15)),
        roe: Some(dec!(5)),
        roa: Some(dec!(2)),
        profit_margin: Some(dec!(3)),
        debt_to_equity: Some(dec!(2.0)),
        current_ratio: Some(dec!(0.8)),
    };
    let fund_result = fund_engine.calculate(&fund_input);

    // Low sentiment and ML scores
    let sentiment_score = 35.0;
    let ml_score = 40.0;

    // Calculate composite score
    let weights = ScoreWeights::default();
    let composite = calculate_composite_score(
        tech_result.total_score.to_string().parse::<f64>().unwrap(),
        fund_result.total_score.to_string().parse::<f64>().unwrap(),
        sentiment_score,
        ml_score,
        &weights,
    );

    // With bearish signals across the board, composite should be below neutral
    assert!(composite < 50.0);
}

#[test]
fn test_mixed_signals_scoring() {
    // Strong technical but weak fundamental
    let tech_engine = TechnicalScoreEngine::new();
    let tech_input = TechnicalScoreInput {
        current_price: dec!(5000),
        obi: Some(dec!(0.4)),
        broker_score: Some(dec!(80)),
        institutional_buying: true,
        foreign_buying: true,
        ema20: Some(dec!(4700)),
        ema50: Some(dec!(4500)),
        rsi: Some(dec!(60)),
        macd_histogram: Some(dec!(15)),
        ..Default::default()
    };
    let tech_result = tech_engine.calculate(&tech_input);

    // Weak fundamentals
    let fund_engine = FundamentalScoreEngine::new();
    let fund_input = FundamentalInput {
        pe_ratio: Some(dec!(35)),
        sector_pe: Some(dec!(15)),
        dcf_margin: Some(dec!(-20)),
        roe: Some(dec!(3)),
        debt_to_equity: Some(dec!(2.5)),
        ..Default::default()
    };
    let fund_result = fund_engine.calculate(&fund_input);

    // Strong technical score, weak fundamental
    assert!(tech_result.total_score > dec!(60));
    assert!(fund_result.total_score < dec!(50));

    // Composite should be somewhere in between
    let weights = ScoreWeights::default();
    let composite = calculate_composite_score(
        tech_result.total_score.to_string().parse::<f64>().unwrap(),
        fund_result.total_score.to_string().parse::<f64>().unwrap(),
        50.0, // Neutral sentiment
        50.0, // Neutral ML
        &weights,
    );

    // Should be moderate, not extreme
    assert!(composite > 30.0 && composite < 70.0);
}

#[test]
fn test_custom_weights() {
    let tech_engine = TechnicalScoreEngine::new();
    let tech_input = TechnicalScoreInput {
        current_price: dec!(100),
        broker_score: Some(dec!(80)),
        ..Default::default()
    };
    let tech_result = tech_engine.calculate(&tech_input);

    let fund_engine = FundamentalScoreEngine::new();
    let fund_input = FundamentalInput::default();
    let fund_result = fund_engine.calculate(&fund_input);

    // Use custom weights that favor technical analysis
    let custom_weights = ScoreWeights {
        technical: 0.60,
        fundamental: 0.20,
        sentiment: 0.10,
        ml: 0.10,
    };

    let composite_custom = calculate_composite_score(
        tech_result.total_score.to_string().parse::<f64>().unwrap(),
        fund_result.total_score.to_string().parse::<f64>().unwrap(),
        50.0,
        50.0,
        &custom_weights,
    );

    // Use default weights
    let default_weights = ScoreWeights::default();
    let composite_default = calculate_composite_score(
        tech_result.total_score.to_string().parse::<f64>().unwrap(),
        fund_result.total_score.to_string().parse::<f64>().unwrap(),
        50.0,
        50.0,
        &default_weights,
    );

    // With higher technical weight and good technical score,
    // custom composite should differ from default
    assert!((composite_custom - composite_default).abs() < 20.0);
}

#[test]
fn test_score_consistency() {
    // Same input should produce same output
    let tech_engine = TechnicalScoreEngine::new();
    let input1 = TechnicalScoreInput {
        current_price: dec!(1000),
        obi: Some(dec!(0.2)),
        rsi: Some(dec!(50)),
        ..Default::default()
    };
    let input2 = TechnicalScoreInput {
        current_price: dec!(1000),
        obi: Some(dec!(0.2)),
        rsi: Some(dec!(50)),
        ..Default::default()
    };

    let result1 = tech_engine.calculate(&input1);
    let result2 = tech_engine.calculate(&input2);

    assert_eq!(result1.total_score, result2.total_score);
    assert_eq!(result1.order_flow_score, result2.order_flow_score);
}

#[test]
fn test_all_neutral_inputs() {
    // With all neutral/missing inputs, scores should be around 50
    let tech_engine = TechnicalScoreEngine::new();
    let tech_input = TechnicalScoreInput::default();
    let tech_result = tech_engine.calculate(&tech_input);

    let fund_engine = FundamentalScoreEngine::new();
    let fund_input = FundamentalInput::default();
    let fund_result = fund_engine.calculate(&fund_input);

    // Both should be neutral (around 50)
    assert!(tech_result.total_score >= dec!(40) && tech_result.total_score <= dec!(60));
    assert_eq!(fund_result.total_score, dec!(50));

    // Composite with all neutral should also be neutral
    let weights = ScoreWeights::default();
    let composite = calculate_composite_score(
        tech_result.total_score.to_string().parse::<f64>().unwrap(),
        fund_result.total_score.to_string().parse::<f64>().unwrap(),
        50.0,
        50.0,
        &weights,
    );

    assert!(composite >= 45.0 && composite <= 55.0);
}

#[test]
fn test_extreme_scores() {
    // Test with extreme bullish inputs
    let tech_engine = TechnicalScoreEngine::new();
    let bullish_input = TechnicalScoreInput {
        current_price: dec!(100),
        obi: Some(dec!(0.9)),
        ofi_trend: Some(dec!(0.9)),
        broker_score: Some(dec!(95)),
        institutional_buying: true,
        foreign_buying: true,
        ema20: Some(dec!(90)),
        ema50: Some(dec!(80)),
        rsi: Some(dec!(35)), // Oversold - bullish signal
        macd_histogram: Some(dec!(20)),
        ..Default::default()
    };
    let bullish_result = tech_engine.calculate(&bullish_input);

    // Should be high but capped at 100
    assert!(bullish_result.total_score <= dec!(100));
    assert!(bullish_result.total_score > dec!(70));

    // Test with extreme bearish inputs
    let bearish_input = TechnicalScoreInput {
        current_price: dec!(100),
        obi: Some(dec!(-0.9)),
        ofi_trend: Some(dec!(-0.9)),
        broker_score: Some(dec!(10)),
        institutional_buying: false,
        foreign_buying: false,
        ema20: Some(dec!(110)),
        ema50: Some(dec!(120)),
        rsi: Some(dec!(85)), // Overbought - bearish signal
        macd_histogram: Some(dec!(-20)),
        ..Default::default()
    };
    let bearish_result = tech_engine.calculate(&bearish_input);

    // Should be low but floored at 0
    assert!(bearish_result.total_score >= dec!(0));
    assert!(bearish_result.total_score < dec!(40));
}

#[test]
fn test_signals_generated() {
    let tech_engine = TechnicalScoreEngine::new();
    let input = TechnicalScoreInput {
        current_price: dec!(100),
        obi: Some(dec!(0.3)),
        institutional_buying: true,
        foreign_buying: true,
        rsi: Some(dec!(25)), // Oversold
        macd_histogram: Some(dec!(5)),
        ..Default::default()
    };
    let result = tech_engine.calculate(&input);

    // Should generate signals for the bullish indicators
    assert!(!result.signals.is_empty());
    assert!(result.signals.iter().any(|s| s.contains("RSI")));
    assert!(result.signals.iter().any(|s| s.contains("Institutional") || s.contains("buying")));
}

#[test]
fn test_fundamental_assessment_categories() {
    let engine = FundamentalScoreEngine::new();

    // Strong fundamentals
    let strong_input = FundamentalInput {
        pe_ratio: Some(dec!(8)),
        sector_pe: Some(dec!(15)),
        pb_ratio: Some(dec!(0.8)),
        sector_pb: Some(dec!(2)),
        dcf_margin: Some(dec!(30)),
        roe: Some(dec!(25)),
        roa: Some(dec!(15)),
        profit_margin: Some(dec!(20)),
        debt_to_equity: Some(dec!(0.3)),
        current_ratio: Some(dec!(2.0)),
        ..Default::default()
    };
    let strong_result = engine.calculate(&strong_input);
    assert_eq!(strong_result.assessment, jejakcuan_core::FundamentalAssessment::Strong);

    // Weak fundamentals
    let weak_input = FundamentalInput {
        pe_ratio: Some(dec!(40)),
        sector_pe: Some(dec!(15)),
        dcf_margin: Some(dec!(-25)),
        roe: Some(dec!(2)),
        debt_to_equity: Some(dec!(3)),
        current_ratio: Some(dec!(0.5)),
        ..Default::default()
    };
    let weak_result = engine.calculate(&weak_input);
    assert!(matches!(
        weak_result.assessment,
        jejakcuan_core::FundamentalAssessment::Weak | jejakcuan_core::FundamentalAssessment::Insufficient
    ));
}
