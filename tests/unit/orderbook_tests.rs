//! Integration tests for the orderbook module.

use option_chain_orderbook::orderbook::{OptionOrderBook, UnderlyingOrderBookManager};
use optionstratlib::{ExpirationDate, OptionStyle, pos};
use orderbook_rs::{OrderId, Side};

#[test]
fn test_option_order_book_integration() {
    let book = OptionOrderBook::new("BTC-20240329-50000-C", OptionStyle::Call);

    // Add orders
    book.add_limit_order(OrderId::new(), Side::Buy, 100, 10)
        .unwrap();
    book.add_limit_order(OrderId::new(), Side::Sell, 101, 5)
        .unwrap();

    // Verify state
    assert_eq!(book.order_count(), 2);
    assert!(book.best_quote().is_two_sided());
}

#[test]
fn test_underlying_manager_integration() {
    let manager = UnderlyingOrderBookManager::new();
    let exp_date = ExpirationDate::Days(pos!(30.0));

    // Create BTC option chain
    {
        let btc = manager.get_or_create("BTC");
        let exp = btc.get_or_create_expiration(exp_date);
        let strike = exp.get_or_create_strike(50000);

        // Add orders to call and put
        strike
            .call()
            .add_limit_order(OrderId::new(), Side::Buy, 100, 10)
            .unwrap();
        strike
            .put()
            .add_limit_order(OrderId::new(), Side::Sell, 50, 5)
            .unwrap();
    }

    // Verify aggregation
    let stats = manager.stats();
    assert_eq!(stats.underlying_count, 1);
    assert_eq!(stats.total_expirations, 1);
    assert_eq!(stats.total_strikes, 1);
    assert_eq!(stats.total_orders, 2);
}
