//! Spread Quoting Example (Avellaneda-Stoikov Model)
//!
//! This example demonstrates the spread calculation using the Avellaneda-Stoikov
//! market making model:
//! - Calculating optimal spreads based on volatility and risk aversion
//! - Inventory skew to manage position risk
//! - Generating two-sided quotes
//! - Adjusting quotes for different market conditions
//!
//! Run with: `cargo run --example spread_quoting`

use option_chain_orderbook::quoting::{QuoteParams, SpreadCalculator};
use rust_decimal_macros::dec;
use tracing::info;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("=== Spread Quoting Example (Avellaneda-Stoikov) ===");

    // Create a spread calculator with custom parameters
    let calculator = SpreadCalculator::new()
        .with_min_spread(dec!(0.001)) // Minimum 0.1% spread
        .with_max_spread(dec!(0.10)); // Maximum 10% spread

    info!("Spread Calculator Configuration:");
    info!("  Min spread: 0.1%");
    info!("  Max spread: 10%");

    // Base case: neutral inventory, moderate volatility
    info!("\n--- Base Case: Neutral Inventory ---");
    let base_params = QuoteParams::new(
        dec!(5.00), // Theoretical price (mid)
        dec!(0),    // Current inventory (0 = neutral)
        dec!(0.30), // Volatility (30%)
        dec!(0.25), // Time to expiry (0.25 years = 3 months)
    );

    let spread = calculator.optimal_spread(&base_params);
    let skew = calculator.inventory_skew(&base_params);
    info!("Theo price: $5.00");
    info!("Inventory: 0 (neutral)");
    info!("Volatility: 30%");
    info!("Time to expiry: 3 months");
    info!("Optimal spread: {:.4}", spread);
    info!("Inventory skew: {:.4}", skew);

    // Generate a full quote
    let quote = calculator.generate_quote(&base_params, 1234567890);
    info!("\nGenerated Quote:");
    info!("  Bid: ${:.4} x {}", quote.bid_price(), quote.bid_size());
    info!("  Ask: ${:.4} x {}", quote.ask_price(), quote.ask_size());

    // Case 2: Long inventory (need to sell)
    info!("\n--- Long Inventory (Need to Sell) ---");
    let long_params = QuoteParams::new(
        dec!(5.00),
        dec!(100), // Long 100 contracts
        dec!(0.30),
        dec!(0.25),
    );

    let spread = calculator.optimal_spread(&long_params);
    let skew = calculator.inventory_skew(&long_params);
    let quote = calculator.generate_quote(&long_params, 1234567890);

    info!("Inventory: +100 (long)");
    info!("Optimal spread: {:.4}", spread);
    info!(
        "Inventory skew: {:.4} (negative = lower prices to attract buyers)",
        skew
    );
    info!("Generated Quote:");
    info!(
        "  Bid: ${:.4} (lowered to discourage more buying)",
        quote.bid_price()
    );
    info!(
        "  Ask: ${:.4} (lowered to encourage selling)",
        quote.ask_price()
    );

    // Case 3: Short inventory (need to buy)
    info!("\n--- Short Inventory (Need to Buy) ---");
    let short_params = QuoteParams::new(
        dec!(5.00),
        dec!(-100), // Short 100 contracts
        dec!(0.30),
        dec!(0.25),
    );

    let skew = calculator.inventory_skew(&short_params);
    let quote = calculator.generate_quote(&short_params, 1234567890);

    info!("Inventory: -100 (short)");
    info!(
        "Inventory skew: {:.4} (positive = higher prices to attract sellers)",
        skew
    );
    info!("Generated Quote:");
    info!(
        "  Bid: ${:.4} (raised to encourage buying)",
        quote.bid_price()
    );
    info!(
        "  Ask: ${:.4} (raised to discourage more selling)",
        quote.ask_price()
    );

    // Case 4: High volatility environment
    info!("\n--- High Volatility Environment ---");
    let high_vol_params = QuoteParams::new(
        dec!(5.00),
        dec!(0),
        dec!(0.80), // 80% volatility
        dec!(0.25),
    );

    // Use a calculator with higher max spread for high vol
    let high_vol_calc = SpreadCalculator::new()
        .with_min_spread(dec!(0.001))
        .with_max_spread(dec!(0.50)); // Allow up to 50% spread

    let spread = high_vol_calc.optimal_spread(&high_vol_params);
    let quote = high_vol_calc.generate_quote(&high_vol_params, 1234567890);

    info!("Volatility: 80%");
    info!("Optimal spread: {:.4} (wider due to higher risk)", spread);
    info!("Generated Quote:");
    info!("  Bid: ${:.4}", quote.bid_price());
    info!("  Ask: ${:.4}", quote.ask_price());

    // Case 5: Near expiration
    info!("\n--- Near Expiration ---");
    let near_expiry_params = QuoteParams::new(
        dec!(5.00),
        dec!(0),
        dec!(0.30),
        dec!(0.01), // 0.01 years = ~3.6 days
    );

    let spread = calculator.optimal_spread(&near_expiry_params);
    let quote = calculator.generate_quote(&near_expiry_params, 1234567890);

    info!("Time to expiry: ~4 days");
    info!("Optimal spread: {:.4} (tighter near expiry)", spread);
    info!("Generated Quote:");
    info!("  Bid: ${:.4}", quote.bid_price());
    info!("  Ask: ${:.4}", quote.ask_price());

    // Case 6: Different risk aversion levels
    info!("\n--- Risk Aversion Comparison ---");
    let conservative =
        QuoteParams::new(dec!(5.00), dec!(0), dec!(0.30), dec!(0.25)).with_risk_aversion(dec!(0.5)); // More risk averse

    let aggressive = QuoteParams::new(dec!(5.00), dec!(0), dec!(0.30), dec!(0.25))
        .with_risk_aversion(dec!(0.05)); // Less risk averse

    let conservative_spread = calculator.optimal_spread(&conservative);
    let aggressive_spread = calculator.optimal_spread(&aggressive);

    info!("Conservative (γ=0.5): spread = {:.4}", conservative_spread);
    info!("Aggressive (γ=0.05): spread = {:.4}", aggressive_spread);
    info!("More risk-averse market makers quote wider spreads");

    // Demonstrate quote stream simulation
    info!("\n--- Quote Stream Simulation ---");
    info!("Simulating quotes as inventory changes...\n");

    let inventories = vec![-50, -25, 0, 25, 50, 75, 100];
    info!(
        "{:>10} {:>10} {:>10} {:>10}",
        "Inventory", "Bid", "Ask", "Mid Shift"
    );

    for inv in inventories {
        let params = QuoteParams::new(dec!(5.00), inv.into(), dec!(0.30), dec!(0.25));
        let quote = calculator.generate_quote(&params, 1234567890);
        let mid = (quote.bid_price() + quote.ask_price()) / dec!(2);
        let shift = mid - dec!(5.00);
        info!(
            "{:>10} {:>10.4} {:>10.4} {:>+10.4}",
            inv,
            quote.bid_price(),
            quote.ask_price(),
            shift
        );
    }

    info!("\n=== Example Complete ===");
}
