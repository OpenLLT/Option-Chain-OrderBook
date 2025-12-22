//! Multi-Book Manager Example
//!
//! This example demonstrates managing multiple order books for an option chain:
//! - Creating and managing order books for multiple strikes
//! - Efficient lookup by symbol
//! - Aggregating statistics across all books
//! - Getting quotes for the entire chain
//!
//! Run with: `cargo run --example multi_book_manager`

use option_chain_orderbook::orderbook::OptionOrderBookManager;
use orderbook_rs::{OrderId, Side};
use tracing::info;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("=== Multi-Book Manager Example ===");

    // Create a manager with pre-allocated capacity for efficiency
    let mut manager = OptionOrderBookManager::with_capacity(100);
    info!("Created order book manager with capacity for 100 books");

    // Define strikes for our option chain (BTC options expiring March 29, 2024)
    let strikes = vec![45000, 47500, 50000, 52500, 55000];
    let expiry = "20240329";

    // Create order books for calls and puts at each strike
    info!("\n--- Creating Order Books ---");
    for strike in &strikes {
        // Create call order book
        let call_symbol = format!("BTC-{}-{}-C", expiry, strike);
        let call_book = manager.get_or_create(&call_symbol);

        // Add some sample orders to the call
        call_book
            .add_limit_order(OrderId::new(), Side::Buy, 100, 10)
            .unwrap();
        call_book
            .add_limit_order(OrderId::new(), Side::Sell, 105, 8)
            .unwrap();

        // Create put order book
        let put_symbol = format!("BTC-{}-{}-P", expiry, strike);
        let put_book = manager.get_or_create(&put_symbol);

        // Add some sample orders to the put
        put_book
            .add_limit_order(OrderId::new(), Side::Buy, 80, 15)
            .unwrap();
        put_book
            .add_limit_order(OrderId::new(), Side::Sell, 85, 12)
            .unwrap();

        info!(
            "Created books for strike {}: {} and {}",
            strike, call_symbol, put_symbol
        );
    }

    // Check manager statistics
    info!("\n--- Manager Statistics ---");
    let stats = manager.stats();
    info!("Total books: {}", stats.book_count);
    info!("Total orders: {}", stats.total_orders);
    info!("Two-sided books: {}", stats.two_sided_count);

    // Look up a specific order book
    info!("\n--- Looking Up Specific Book ---");
    let target_symbol = "BTC-20240329-50000-C";
    if let Some(book) = manager.get(target_symbol) {
        let quote = book.best_quote();
        info!("Found book: {}", book.symbol());
        info!(
            "  Quote: {} @ {:?} / {} @ {:?}",
            quote.bid_size(),
            quote.bid_price(),
            quote.ask_size(),
            quote.ask_price()
        );
    }

    // Get all quotes across the chain
    info!("\n--- All Quotes in Chain ---");
    let all_quotes = manager.all_quotes();
    for (symbol, quote) in all_quotes.iter().take(6) {
        // Show first 6
        if quote.is_two_sided() {
            info!(
                "{}: {} @ {:?} / {} @ {:?} (spread: {:?})",
                symbol,
                quote.bid_size(),
                quote.bid_price(),
                quote.ask_size(),
                quote.ask_price(),
                quote.spread()
            );
        } else {
            info!("{}: one-sided or empty", symbol);
        }
    }

    // Iterate over all books
    info!("\n--- Iterating Over All Books ---");
    let symbols: Vec<_> = manager.symbols().collect();
    info!("Total symbols in manager: {}", symbols.len());
    info!("First 4 symbols: {:?}", &symbols[..4.min(symbols.len())]);

    // Demonstrate removing a book
    info!("\n--- Removing a Book ---");
    let removed = manager.remove("BTC-20240329-45000-P");
    info!(
        "Removed BTC-20240329-45000-P: {}",
        if removed.is_some() { "yes" } else { "no" }
    );
    info!("Books remaining: {}", manager.len());

    info!("\n=== Example Complete ===");
}
