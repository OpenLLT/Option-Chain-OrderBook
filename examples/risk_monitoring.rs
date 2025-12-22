//! Risk Monitoring Example
//!
//! This example demonstrates the risk controller functionality:
//! - Setting up risk limits (Greeks, P&L, position)
//! - Monitoring for limit breaches
//! - Automatic trading halts
//! - P&L tracking and drawdown monitoring
//!
//! Run with: `cargo run --example risk_monitoring`

use option_chain_orderbook::pricing::Greeks;
use option_chain_orderbook::risk::{RiskController, RiskLimits};
use rust_decimal_macros::dec;
use tracing::info;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("=== Risk Monitoring Example ===");

    // Create risk limits
    let limits = RiskLimits {
        max_delta: dec!(1000),
        max_gamma: dec!(100),
        max_vega: dec!(5000),
        max_daily_loss: dec!(50000),
        max_drawdown: dec!(25000),
        max_position_value: dec!(500000),
    };

    info!("Risk Limits Configuration:");
    info!("  Max Delta: {}", limits.max_delta);
    info!("  Max Gamma: {}", limits.max_gamma);
    info!("  Max Vega: {}", limits.max_vega);
    info!("  Max Daily Loss: ${}", limits.max_daily_loss);
    info!("  Max Drawdown: ${}", limits.max_drawdown);
    info!("  Max Position Value: ${}", limits.max_position_value);

    // Create risk controller
    let mut controller = RiskController::new(limits);
    info!("\nRisk controller initialized");
    info!("Trading halted: {}", controller.is_halted());

    // Scenario 1: Normal operation - within limits
    info!("\n--- Scenario 1: Normal Operation ---");
    let normal_greeks = Greeks::new(dec!(500), dec!(30), dec!(-50), dec!(2000), dec!(100));

    let breaches = controller.check_greek_limits(&normal_greeks);
    info!("Portfolio Greeks:");
    info!("  Delta: {}", normal_greeks.delta());
    info!("  Gamma: {}", normal_greeks.gamma());
    info!("  Vega: {}", normal_greeks.vega());
    info!("Limit breaches: {}", breaches.len());
    if breaches.is_empty() {
        info!("All Greek limits OK ✓");
    }

    // Update P&L
    controller.update_pnl(dec!(5000)); // Made $5,000
    info!("\nP&L updated: +$5,000");
    info!("Trading halted: {}", controller.is_halted());

    // Scenario 2: Delta limit breach
    info!("\n--- Scenario 2: Delta Limit Breach ---");
    let high_delta = Greeks::new(dec!(1500), dec!(50), dec!(-80), dec!(3000), dec!(150));

    let breaches = controller.check_greek_limits(&high_delta);
    info!("Portfolio Delta: {} (limit: 1000)", high_delta.delta());
    info!("Limit breaches: {}", breaches.len());
    for breach in &breaches {
        info!("  ⚠ {:?}", breach);
    }

    // Scenario 3: Multiple limit breaches
    info!("\n--- Scenario 3: Multiple Limit Breaches ---");
    let risky_greeks = Greeks::new(dec!(1200), dec!(150), dec!(-100), dec!(6000), dec!(200));

    let breaches = controller.check_greek_limits(&risky_greeks);
    info!("Portfolio Greeks:");
    info!("  Delta: {} (limit: 1000)", risky_greeks.delta());
    info!("  Gamma: {} (limit: 100)", risky_greeks.gamma());
    info!("  Vega: {} (limit: 5000)", risky_greeks.vega());
    info!("\nBreaches detected: {}", breaches.len());
    for breach in &breaches {
        info!("  ⚠ {:?}", breach);
    }

    // Scenario 4: P&L loss triggers halt
    info!("\n--- Scenario 4: Loss Limit Breach ---");
    controller.update_pnl(dec!(-55000)); // Lost $55,000
    info!("P&L updated: -$55,000 (limit: -$50,000)");
    info!("Trading halted: {}", controller.is_halted());
    if let Some(reason) = controller.halt_reason() {
        info!("Halt reason: {}", reason);
    }

    // Try to resume trading
    info!("\n--- Attempting to Resume ---");
    controller.resume();
    info!("Called resume()");
    info!("Trading halted: {}", controller.is_halted());

    // Scenario 5: Drawdown monitoring
    info!("\n--- Scenario 5: Drawdown Monitoring ---");
    let mut dd_controller = RiskController::new(limits);

    // Simulate P&L sequence: profit then loss
    let pnl_sequence = vec![
        (10000, "Made $10,000"),
        (20000, "Made $20,000 (peak)"),
        (15000, "Lost $5,000 (drawdown: $5,000)"),
        (5000, "Lost $10,000 (drawdown: $15,000)"),
        (-5000, "Lost $10,000 (drawdown: $25,000)"),
        (-10000, "Lost $5,000 (drawdown: $30,000 - BREACH!)"),
    ];

    info!("P&L Sequence:");
    for (pnl, description) in pnl_sequence {
        dd_controller.update_pnl(pnl.into());
        let status = if dd_controller.is_halted() {
            "HALTED"
        } else {
            "OK"
        };
        info!("  {} -> {}", description, status);
    }

    if let Some(reason) = dd_controller.halt_reason() {
        info!("\nHalt reason: {}", reason);
    }

    // Scenario 6: Position value limit
    info!("\n--- Scenario 6: Position Value Monitoring ---");
    let mut pos_controller = RiskController::new(limits);

    pos_controller.update_position_value(dec!(400000));
    info!("Position value: $400,000 (limit: $500,000)");
    info!("Trading halted: {}", pos_controller.is_halted());

    pos_controller.update_position_value(dec!(600000));
    info!("Position value: $600,000 (limit: $500,000)");
    info!("Trading halted: {}", pos_controller.is_halted());
    if let Some(reason) = pos_controller.halt_reason() {
        info!("Halt reason: {}", reason);
    }

    // Scenario 7: Daily reset
    info!("\n--- Scenario 7: Daily Reset ---");
    let mut daily_controller = RiskController::new(limits);
    daily_controller.update_pnl(dec!(-30000));
    info!("End of day P&L: -$30,000");
    info!("Trading halted: {}", daily_controller.is_halted());

    daily_controller.reset_daily();
    info!("\nNew trading day - reset called");
    info!("Trading halted: {}", daily_controller.is_halted());

    // Using default limits
    info!("\n--- Default Risk Limits ---");
    let default_limits = RiskLimits::default();
    info!("Default max delta: {}", default_limits.max_delta);
    info!("Default max gamma: {}", default_limits.max_gamma);
    info!("Default max vega: {}", default_limits.max_vega);
    info!("Default max daily loss: ${}", default_limits.max_daily_loss);

    // Risk monitoring summary
    info!("\n--- Risk Monitoring Best Practices ---");
    info!("1. Check Greek limits before placing new orders");
    info!("2. Update P&L after each trade execution");
    info!("3. Monitor position value continuously");
    info!("4. Implement automatic position reduction on breaches");
    info!("5. Reset daily limits at start of each trading day");
    info!("6. Log all halt events for review");

    info!("\n=== Example Complete ===");
}
