//! Market Making Simulation Example
//!
//! This example demonstrates a complete market making workflow combining
//! all components of the library.
//!
//! Run with: `cargo run --example market_making_simulation`

use option_chain_orderbook::hedging::{DeltaHedger, HedgeParams};
use option_chain_orderbook::inventory::{InventoryManager, PositionLimits};
use option_chain_orderbook::orderbook::OptionOrderBookManager;
use option_chain_orderbook::pricing::Greeks;
use option_chain_orderbook::quoting::{QuoteParams, SpreadCalculator};
use option_chain_orderbook::risk::{RiskController, RiskLimits};
use orderbook_rs::{OrderId, Side};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;
use tracing::info;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("=== Market Making Simulation ===");

    // Initialize components
    let pos_limits = PositionLimits::new(dec!(100), dec!(500), dec!(2000), dec!(5000));
    let risk_limits = RiskLimits {
        max_delta: dec!(500),
        max_gamma: dec!(50),
        max_vega: dec!(2000),
        max_daily_loss: dec!(10000),
        max_drawdown: dec!(5000),
        max_position_value: dec!(100000),
    };
    let hedge_params = HedgeParams {
        target_delta: dec!(0),
        hedge_threshold: dec!(100),
        min_hedge_size: dec!(10),
        max_hedge_size: dec!(200),
        ..HedgeParams::default()
    };

    let mut books = OptionOrderBookManager::with_capacity(50);
    let mut inventory = InventoryManager::new("BTC", pos_limits, dec!(1));
    let spread_calc = SpreadCalculator::new()
        .with_min_spread(dec!(0.005))
        .with_max_spread(dec!(0.15));
    let mut hedger = DeltaHedger::new(hedge_params);
    let risk = RiskController::new(risk_limits);
    let spot_price = dec!(50000);
    let mut timestamp: u64 = 1000000;

    // Option chain definition
    let options = vec![
        ("BTC-20240329-50000-C", dec!(2.00), dec!(0.30)),
        ("BTC-20240329-52000-C", dec!(1.00), dec!(0.32)),
        ("BTC-20240329-50000-P", dec!(2.00), dec!(0.30)),
    ];

    // Round 1: Generate initial quotes
    info!("--- Round 1: Initial Quotes ---");
    for (symbol, theo, vol) in &options {
        let inv = inventory
            .get_position(symbol)
            .map(|p| p.quantity())
            .unwrap_or(dec!(0));
        let params = QuoteParams::new(*theo, inv, *vol, dec!(0.25));
        let quote = spread_calc.generate_quote(&params, timestamp);
        let book = books.get_or_create(*symbol);
        let bid = (quote.bid_price() * dec!(100)).to_u64().unwrap_or(0);
        let ask = (quote.ask_price() * dec!(100)).to_u64().unwrap_or(0);
        let _ = book.add_limit_order(OrderId::new(), Side::Buy, bid, 10);
        let _ = book.add_limit_order(OrderId::new(), Side::Sell, ask, 10);
        info!("  {} | Bid: {} | Ask: {}", symbol, bid, ask);
    }

    // Round 2: Simulate fills
    info!("\n--- Round 2: Customer Trades ---");
    timestamp += 1000;

    // Customer buys calls from us (we sell)
    let _ = inventory.record_trade("BTC-20240329-50000-C", dec!(-20), dec!(2.05), timestamp);
    info!("  SOLD 20 BTC-20240329-50000-C @ $2.05");

    // Customer sells puts to us (we buy)
    let _ = inventory.record_trade("BTC-20240329-50000-P", dec!(15), dec!(1.95), timestamp);
    info!("  BOUGHT 15 BTC-20240329-50000-P @ $1.95");

    // Update Greeks
    for (symbol, _, _) in &options {
        if let Some(pos) = inventory.get_position_mut(symbol) {
            let delta = if symbol.ends_with("-C") {
                dec!(0.5)
            } else {
                dec!(-0.5)
            };
            let greeks = Greeks::new(
                delta * pos.quantity(),
                dec!(0.02) * pos.quantity().abs(),
                dec!(-0.05) * pos.quantity().abs(),
                dec!(0.15) * pos.quantity().abs(),
                dec!(0.08) * pos.quantity().abs(),
            );
            pos.update_greeks(greeks, timestamp);
        }
    }

    // Round 3: Check portfolio
    info!("\n--- Round 3: Portfolio State ---");
    let greeks = inventory.total_greeks();
    info!("  Delta: {}", greeks.delta());
    info!("  Gamma: {}", greeks.gamma());
    info!("  Vega: {}", greeks.vega());

    // Round 4: Check hedges
    info!("\n--- Round 4: Hedge Check ---");
    hedger.update_delta(&greeks);
    if let Some(order) = hedger.calculate_hedge("BTC", spot_price, timestamp) {
        info!(
            "  HEDGE: {} {} BTC",
            if order.quantity > Decimal::ZERO {
                "BUY"
            } else {
                "SELL"
            },
            order.quantity.abs()
        );
    } else {
        info!("  No hedge needed");
    }

    // Round 5: Risk check
    info!("\n--- Round 5: Risk Check ---");
    let breaches = risk.check_greek_limits(&greeks);
    if breaches.is_empty() {
        info!("  All limits OK ✓");
    } else {
        for b in &breaches {
            info!("  ⚠ {:?}", b);
        }
    }

    // Round 6: Re-quote with inventory skew
    info!("\n--- Round 6: Updated Quotes ---");
    timestamp += 1000;
    for (symbol, theo, vol) in &options {
        let inv = inventory
            .get_position(symbol)
            .map(|p| p.quantity())
            .unwrap_or(dec!(0));
        let params = QuoteParams::new(*theo, inv, *vol, dec!(0.25));
        let quote = spread_calc.generate_quote(&params, timestamp);
        let bid = (quote.bid_price() * dec!(100)).to_u64().unwrap_or(0);
        let ask = (quote.ask_price() * dec!(100)).to_u64().unwrap_or(0);
        info!(
            "  {} | Inv: {:>3} | Bid: {} | Ask: {}",
            symbol, inv, bid, ask
        );
    }

    // Summary
    info!("\n--- Summary ---");
    let stats = books.stats();
    info!("  Books: {}", stats.book_count);
    info!("  Orders: {}", stats.total_orders);
    info!("  Positions: {}", inventory.position_count());

    info!("\n=== Simulation Complete ===");
}
