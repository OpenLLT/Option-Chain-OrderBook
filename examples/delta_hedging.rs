//! Delta Hedging Example
//!
//! This example demonstrates automated delta hedging:
//! - Configuring hedge parameters (thresholds, bands, sizes)
//! - Tracking portfolio delta
//! - Generating hedge orders when thresholds are breached
//! - Different hedging strategies
//!
//! Run with: `cargo run --example delta_hedging`

use option_chain_orderbook::hedging::{DeltaHedger, HedgeParams};
use option_chain_orderbook::pricing::Greeks;
use rust_decimal_macros::dec;
use tracing::info;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("=== Delta Hedging Example ===");

    // Create hedge parameters
    let params = HedgeParams {
        target_delta: dec!(0),      // Target delta (0 = delta neutral)
        hedge_threshold: dec!(50),  // Hedge threshold (hedge when |delta| > 50)
        min_hedge_size: dec!(10),   // Min hedge size
        max_hedge_size: dec!(1000), // Max hedge size
        use_limit_orders: true,     // Use limit orders
        limit_offset_bps: dec!(10), // 10 bps offset
    };

    info!("Hedge Parameters:");
    info!("  Target delta: {}", params.target_delta);
    info!("  Hedge threshold: {}", params.hedge_threshold);
    info!("  Min hedge size: {}", params.min_hedge_size);
    info!("  Max hedge size: {}", params.max_hedge_size);
    info!("  Use limit orders: {}", params.use_limit_orders);
    info!("  Limit offset: {} bps", params.limit_offset_bps);

    // Create the hedger
    let mut hedger = DeltaHedger::new(params);
    info!("\nCreated delta hedger");

    // Scenario 1: Small delta - no hedge needed
    info!("\n--- Scenario 1: Small Delta ---");
    let small_delta = Greeks::new(dec!(30), dec!(2), dec!(-5), dec!(100), dec!(10));
    hedger.update_delta(&small_delta);

    info!("Portfolio delta: {}", hedger.current_delta());
    info!("Delta deviation: {}", hedger.delta_deviation());
    info!("Needs hedge: {}", hedger.needs_hedge());

    let hedge = hedger.calculate_hedge("BTC", dec!(50000), 1000000);
    match hedge {
        Some(order) => info!("Hedge order: {:?}", order),
        None => info!("No hedge needed"),
    }

    // Scenario 2: Large positive delta - need to sell
    info!("\n--- Scenario 2: Large Positive Delta ---");
    let large_positive = Greeks::new(dec!(150), dec!(5), dec!(-10), dec!(200), dec!(20));
    hedger.update_delta(&large_positive);

    info!("Portfolio delta: {}", hedger.current_delta());
    info!("Delta deviation: {}", hedger.delta_deviation());
    info!("Needs hedge: {}", hedger.needs_hedge());

    if let Some(order) = hedger.calculate_hedge("BTC", dec!(50000), 1000001) {
        info!("\nHedge Order Generated:");
        info!("  Symbol: {}", order.symbol);
        info!("  Quantity: {} (negative = sell)", order.quantity);
        info!("  Limit price: {:?}", order.limit_price);
        info!("  Reason: {:?}", order.reason);
        info!("  Timestamp: {}", order.timestamp_ms);
    }

    // Scenario 3: Large negative delta - need to buy
    info!("\n--- Scenario 3: Large Negative Delta ---");
    let large_negative = Greeks::new(dec!(-200), dec!(4), dec!(-8), dec!(180), dec!(15));
    hedger.update_delta(&large_negative);

    info!("Portfolio delta: {}", hedger.current_delta());
    info!("Delta deviation: {}", hedger.delta_deviation());

    if let Some(order) = hedger.calculate_hedge("BTC", dec!(50000), 1000002) {
        info!("\nHedge Order Generated:");
        info!("  Symbol: {}", order.symbol);
        info!("  Quantity: {} (positive = buy)", order.quantity);
        info!("  Limit price: {:?}", order.limit_price);
        info!("  Reason: {:?}", order.reason);
    }

    // Scenario 4: Very large delta - capped at max size
    info!("\n--- Scenario 4: Very Large Delta (Capped) ---");
    let very_large = Greeks::new(dec!(5000), dec!(10), dec!(-20), dec!(500), dec!(50));
    hedger.update_delta(&very_large);

    info!("Portfolio delta: {}", hedger.current_delta());
    info!("Delta deviation: {}", hedger.delta_deviation());

    if let Some(order) = hedger.calculate_hedge("BTC", dec!(50000), 1000003) {
        info!("\nHedge Order Generated:");
        info!("  Quantity: {} (capped at max_hedge_size)", order.quantity);
        info!(
            "  Note: Full hedge would require {}",
            -hedger.delta_deviation()
        );
    }

    // Demonstrate different hedging configurations
    info!("\n--- Different Hedging Strategies ---");

    // Aggressive hedger (tight threshold)
    let aggressive_params = HedgeParams {
        target_delta: dec!(0),
        hedge_threshold: dec!(20),
        min_hedge_size: dec!(5),
        max_hedge_size: dec!(500),
        ..HedgeParams::default()
    };
    let mut aggressive = DeltaHedger::new(aggressive_params);
    aggressive.update_delta(&Greeks::new(dec!(25), dec!(1), dec!(-2), dec!(50), dec!(5)));
    info!(
        "Aggressive (threshold=20): needs_hedge={}",
        aggressive.needs_hedge()
    );

    // Conservative hedger (wide threshold)
    let conservative_params = HedgeParams {
        target_delta: dec!(0),
        hedge_threshold: dec!(100),
        min_hedge_size: dec!(20),
        max_hedge_size: dec!(2000),
        ..HedgeParams::default()
    };
    let mut conservative = DeltaHedger::new(conservative_params);
    conservative.update_delta(&Greeks::new(
        dec!(80),
        dec!(3),
        dec!(-6),
        dec!(150),
        dec!(12),
    ));
    info!(
        "Conservative (threshold=100): needs_hedge={}",
        conservative.needs_hedge()
    );

    // Non-zero target delta (directional bias)
    info!("\n--- Directional Hedging (Target Delta = +50) ---");
    let directional_params = HedgeParams {
        target_delta: dec!(50), // Target: stay long 50 delta
        hedge_threshold: dec!(30),
        min_hedge_size: dec!(10),
        max_hedge_size: dec!(500),
        ..HedgeParams::default()
    };
    let mut directional = DeltaHedger::new(directional_params);

    // With delta at 100, deviation is 50, which exceeds threshold
    directional.update_delta(&Greeks::new(
        dec!(100),
        dec!(3),
        dec!(-5),
        dec!(120),
        dec!(10),
    ));
    info!("Current delta: {}", directional.current_delta());
    info!("Target delta: 50");
    info!("Deviation from target: {}", directional.delta_deviation());
    info!("Needs hedge: {}", directional.needs_hedge());

    if let Some(order) = directional.calculate_hedge("BTC", dec!(50000), 1000004) {
        info!("Hedge to reach target: {} units", order.quantity);
    }

    // Simulation: Delta changes over time
    info!("\n--- Delta Evolution Simulation ---");
    let sim_params = HedgeParams {
        target_delta: dec!(0),
        hedge_threshold: dec!(50),
        min_hedge_size: dec!(10),
        max_hedge_size: dec!(500),
        ..HedgeParams::default()
    };
    let mut sim_hedger = DeltaHedger::new(sim_params);

    let delta_sequence = [20, 45, 60, 35, -30, -70, -40, 10];
    info!(
        "{:>10} {:>15} {:>12} {:>15}",
        "Delta", "Deviation", "Needs Hedge", "Action"
    );

    for (i, delta) in delta_sequence.iter().enumerate() {
        let greeks = Greeks::new((*delta).into(), dec!(2), dec!(-3), dec!(80), dec!(8));
        sim_hedger.update_delta(&greeks);

        let needs = sim_hedger.needs_hedge();
        let action = if needs {
            if let Some(order) = sim_hedger.calculate_hedge("BTC", dec!(50000), 1000000 + i as u64)
            {
                format!("Hedge {}", order.quantity)
            } else {
                "None".to_string()
            }
        } else {
            "Hold".to_string()
        };

        info!(
            "{:>10} {:>15} {:>12} {:>15}",
            delta,
            sim_hedger.delta_deviation(),
            needs,
            action
        );
    }

    info!("\n=== Example Complete ===");
}
