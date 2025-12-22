//! Inventory Management Example
//!
//! This example demonstrates position tracking and inventory management:
//! - Creating and tracking positions
//! - Recording trades and updating average prices
//! - Calculating P&L (realized and unrealized)
//! - Managing position limits
//! - Aggregating Greeks across positions
//!
//! Run with: `cargo run --example inventory_management`

use option_chain_orderbook::inventory::{InventoryManager, Position, PositionLimits};
use option_chain_orderbook::pricing::Greeks;
use rust_decimal_macros::dec;
use tracing::info;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("=== Inventory Management Example ===");

    // Create position limits
    let limits = PositionLimits::new(
        dec!(100),   // Max 100 contracts per option
        dec!(500),   // Max 500 contracts per strike
        dec!(2000),  // Max 2000 contracts per expiration
        dec!(10000), // Max 10000 contracts total
    );

    info!("Position Limits:");
    info!("  Per option: {}", limits.per_option());
    info!("  Per strike: {}", limits.per_strike());
    info!("  Per expiration: {}", limits.per_expiration());
    info!("  Per underlying: {}", limits.per_underlying());

    // Create an inventory manager for BTC options
    let mut manager = InventoryManager::new("BTC", limits, dec!(1)); // Multiplier of 1
    info!("\nCreated inventory manager for: {}", manager.underlying());

    // Simulate trading activity
    info!("\n--- Recording Trades ---");

    // Trade 1: Buy 10 BTC-50000-C at $500
    let symbol1 = "BTC-20240329-50000-C";
    manager
        .record_trade(symbol1, dec!(10), dec!(500), 1000000)
        .unwrap();
    info!("Bought 10 {} @ $500", symbol1);

    // Trade 2: Buy 5 more at $520 (average up)
    manager
        .record_trade(symbol1, dec!(5), dec!(520), 1000001)
        .unwrap();
    info!("Bought 5 {} @ $520", symbol1);

    // Trade 3: Buy 20 BTC-50000-P at $300
    let symbol2 = "BTC-20240329-50000-P";
    manager
        .record_trade(symbol2, dec!(20), dec!(300), 1000002)
        .unwrap();
    info!("Bought 20 {} @ $300", symbol2);

    // Trade 4: Sell 8 of the calls at $550 (partial close)
    manager
        .record_trade(symbol1, dec!(-8), dec!(550), 1000003)
        .unwrap();
    info!("Sold 8 {} @ $550", symbol1);

    // Check positions
    info!("\n--- Current Positions ---");
    if let Some(pos) = manager.get_position(symbol1) {
        info!("{}:", symbol1);
        info!("  Quantity: {}", pos.quantity());
        info!("  Average price: ${}", pos.average_price());
        info!("  Cost basis: ${}", pos.cost_basis());
        info!("  Realized P&L: ${}", pos.realized_pnl());

        // Calculate unrealized P&L at current price
        let current_price = dec!(540);
        let unrealized = pos.unrealized_pnl(current_price);
        let total = pos.total_pnl(current_price);
        info!("  Current price: ${}", current_price);
        info!("  Unrealized P&L: ${}", unrealized);
        info!("  Total P&L: ${}", total);
    }

    if let Some(pos) = manager.get_position(symbol2) {
        info!("\n{}:", symbol2);
        info!("  Quantity: {}", pos.quantity());
        info!("  Average price: ${}", pos.average_price());
    }

    // Update Greeks for positions
    info!("\n--- Updating Greeks ---");
    if let Some(pos) = manager.get_position_mut(symbol1) {
        pos.update_greeks(
            Greeks::new(dec!(0.55), dec!(0.02), dec!(-0.05), dec!(0.15), dec!(0.08)),
            1000004,
        );
        info!("Updated Greeks for {}", symbol1);
    }

    if let Some(pos) = manager.get_position_mut(symbol2) {
        pos.update_greeks(
            Greeks::new(
                dec!(-0.45),
                dec!(0.018),
                dec!(-0.04),
                dec!(0.12),
                dec!(-0.06),
            ),
            1000004,
        );
        info!("Updated Greeks for {}", symbol2);
    }

    // Get total portfolio Greeks
    info!("\n--- Portfolio Greeks ---");
    let total_greeks = manager.total_greeks();
    info!("Total Delta: {}", total_greeks.delta());
    info!("Total Gamma: {}", total_greeks.gamma());
    info!("Total Theta: {}", total_greeks.theta());
    info!("Total Vega: {}", total_greeks.vega());

    // Check position limits
    info!("\n--- Limit Checks ---");
    let spot = dec!(50000);
    let multiplier = dec!(1);
    let breaches = manager.check_greek_limits(spot, multiplier);
    if breaches.is_empty() {
        info!("All Greek limits OK");
    } else {
        for breach in &breaches {
            info!("Limit breach: {:?}", breach);
        }
    }

    // Demonstrate Position struct directly
    info!("\n--- Direct Position Usage ---");
    let mut direct_pos = Position::with_entry(dec!(10), dec!(100), dec!(100), 2000000);
    info!(
        "Created position: {} @ ${}",
        direct_pos.quantity(),
        direct_pos.average_price()
    );

    // Add to position
    direct_pos.add(dec!(5), dec!(110), 2000001);
    info!(
        "After adding 5 @ $110: {} @ ${:.2}",
        direct_pos.quantity(),
        direct_pos.average_price()
    );

    // Reduce position
    direct_pos.reduce(dec!(8), dec!(120), 2000002);
    info!(
        "After selling 8 @ $120: {} @ ${:.2}, realized P&L: ${}",
        direct_pos.quantity(),
        direct_pos.average_price(),
        direct_pos.realized_pnl()
    );

    // Portfolio summary
    info!("\n--- Portfolio Summary ---");
    info!("Total positions: {}", manager.position_count());
    info!("Underlying: {}", manager.underlying());

    info!("\n=== Example Complete ===");
}
