//! Greeks Calculation Example
//!
//! This example demonstrates working with option Greeks:
//! - Creating Greeks containers
//! - Arithmetic operations (add, scale, multiply)
//! - Dollar-value calculations
//! - Portfolio aggregation
//!
//! Run with: `cargo run --example greeks_calculation`

use option_chain_orderbook::pricing::Greeks;
use rust_decimal_macros::dec;
use tracing::info;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("=== Greeks Calculation Example ===");

    // Create Greeks for a long call position
    // Delta: 0.55 (55 delta call)
    // Gamma: 0.02 (gamma per 1 point move)
    // Theta: -0.05 (loses $0.05 per day)
    // Vega: 0.15 (gains $0.15 per 1% vol increase)
    // Rho: 0.08 (gains $0.08 per 1% rate increase)
    let long_call = Greeks::new(dec!(0.55), dec!(0.02), dec!(-0.05), dec!(0.15), dec!(0.08));
    info!("Long Call Greeks:");
    info!("  Delta: {}", long_call.delta());
    info!("  Gamma: {}", long_call.gamma());
    info!("  Theta: {}", long_call.theta());
    info!("  Vega:  {}", long_call.vega());
    info!("  Rho:   {}", long_call.rho());

    // Create Greeks for a short put position (negative quantities = short)
    let short_put = Greeks::new(
        dec!(0.35),
        dec!(0.015),
        dec!(0.03),
        dec!(-0.10),
        dec!(-0.05),
    );
    info!("\nShort Put Greeks:");
    info!("  Delta: {}", short_put.delta());
    info!("  Gamma: {}", short_put.gamma());
    info!("  Theta: {}", short_put.theta());
    info!("  Vega:  {}", short_put.vega());
    info!("  Rho:   {}", short_put.rho());

    // Combine positions (portfolio aggregation)
    let portfolio = long_call + short_put;
    info!("\nCombined Portfolio Greeks:");
    info!("  Delta: {}", portfolio.delta());
    info!("  Gamma: {}", portfolio.gamma());
    info!("  Theta: {}", portfolio.theta());
    info!("  Vega:  {}", portfolio.vega());
    info!("  Rho:   {}", portfolio.rho());

    // Scale Greeks for position size (e.g., 10 contracts)
    let contracts = dec!(10);
    let scaled = long_call.scale(contracts);
    info!("\nScaled Long Call (10 contracts):");
    info!("  Delta: {}", scaled.delta());
    info!("  Gamma: {}", scaled.gamma());
    info!("  Theta: {}", scaled.theta());
    info!("  Vega:  {}", scaled.vega());

    // Calculate dollar values
    let spot_price = dec!(50000); // BTC at $50,000
    let multiplier = dec!(1); // Crypto options typically have multiplier of 1

    info!("\n--- Dollar Value Calculations ---");
    info!("Spot price: ${}", spot_price);
    info!("Multiplier: {}", multiplier);

    let dollar_delta = scaled.dollar_delta(spot_price, multiplier);
    info!("\nDollar Delta: ${}", dollar_delta);
    info!(
        "  (Position gains/loses ${} per $1 move in underlying)",
        dollar_delta
    );

    let dollar_gamma = scaled.dollar_gamma(spot_price, multiplier);
    info!("\nDollar Gamma: ${}", dollar_gamma);
    info!("  (Delta changes by ${} per 1% move)", dollar_gamma);

    let dollar_vega = scaled.dollar_vega(multiplier);
    info!("\nDollar Vega: ${}", dollar_vega);
    info!(
        "  (Position gains/loses ${} per 1% vol change)",
        dollar_vega
    );

    let dollar_theta = scaled.dollar_theta(multiplier);
    info!("\nDollar Theta: ${}", dollar_theta);
    info!("  (Position loses ${} per day)", dollar_theta.abs());

    // Demonstrate negation (for reversing positions)
    let reversed = -long_call;
    info!("\n--- Reversed Position (Short Call) ---");
    info!("  Delta: {}", reversed.delta());
    info!("  Gamma: {}", reversed.gamma());
    info!("  Theta: {}", reversed.theta());

    // Check if Greeks are zero
    let zero_greeks = Greeks::zero();
    info!("\n--- Zero Greeks Check ---");
    info!("Zero Greeks is_zero: {}", zero_greeks.is_zero());
    info!("Long Call is_zero: {}", long_call.is_zero());

    // Portfolio of multiple positions
    info!("\n--- Multi-Position Portfolio ---");
    let positions = vec![
        ("BTC-50000-C Long 5", long_call.scale(dec!(5))),
        ("BTC-52000-C Short 3", (-long_call).scale(dec!(3))),
        ("BTC-48000-P Short 2", short_put.scale(dec!(2))),
    ];

    let mut total = Greeks::zero();
    for (name, greeks) in &positions {
        info!("{}: delta={}", name, greeks.delta());
        total = total + *greeks;
    }
    info!("\nTotal Portfolio Delta: {}", total.delta());
    info!("Total Portfolio Gamma: {}", total.gamma());
    info!("Total Portfolio Vega: {}", total.vega());
    info!("Total Portfolio Theta: {}", total.theta());

    // Delta exposure analysis
    info!("\n--- Delta Exposure Analysis ---");
    if total.is_long_delta() {
        info!("Portfolio is LONG delta (benefits from price increase)");
    } else if total.is_short_delta() {
        info!("Portfolio is SHORT delta (benefits from price decrease)");
    } else {
        info!("Portfolio is DELTA NEUTRAL");
    }
    info!("Absolute delta exposure: {}", total.abs_delta());

    info!("\n=== Example Complete ===");
}
